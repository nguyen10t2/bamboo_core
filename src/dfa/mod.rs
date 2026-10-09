//! DFA-based engine for high-performance Vietnamese input method.
//!
//! This module provides the Deterministic Finite Automaton (DFA) state management,
//! bitset fast-path transition lookups, arena allocation for transformations,
//! and the JIT compiler for pre-compiling common syllable transitions.

pub mod flattener;

use crate::engine::Transformation;
use crate::input_method::InputMethod;
use rustc_hash::FxHashMap;

/// Maximum transitions per DFA state. Vietnamese input typically uses ~12 keys per state.
const MAX_TRANS: usize = 16;

/// Maximum DFA state ID that fits in `trans_states: [u16; 16]`.
const MAX_STATE_ID: u32 = 0xFFFF;

/// Hard cap on the number of DFA states (including state 0).
///
/// Bounds long-lived Engine memory: `8192*88B states + ~8192*~5*16B arena +
/// flat + map ≈ 1.5–2MB used`. Valid Vietnamese needs ~4k states
/// (`fc*vowel*tone`), so 8192 leaves 2x headroom while freezing adversarial
/// unique spam (which previously grew unbounded past 1M states / 200MB+).
/// When full, [`Dfa::add_state`] returns 0 for new compositions (dedup hits
/// still succeed) and callers fall back to the rule-engine slow path, which
/// stays correct but slower for the long tail.
pub const DFA_MAX_STATES: usize = 8192;

/// A compact DFA state representing a unique syllable composition.
///
/// Transitions are stored as sorted `(key, state_id)` pairs instead of a full
/// 128-entry table. A 128-bit bitset enables $O(1)$ "has transition?" checks
/// and fast rejection for keys that don't have transitions.
///
/// Memory layout is carefully aligned so that the hot fields (bitset, keys,
/// state IDs) reside within the first 64-byte cache line. Cold fields
/// (composition hash, arena offset, lengths) occupy the tail.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct State {
    /// 128-bit bitset: bit `i = 1` means key `i` has a transition (16 bytes: offset 0..16).
    pub bitset: [u64; 2],
    /// Keys triggering transitions (16 bytes: offset 16..32).
    pub trans_keys: [u8; MAX_TRANS],
    /// Destination state IDs for transitions (32 bytes: offset 32..64).
    pub trans_states: [u16; MAX_TRANS],
    /// Precomputed hash of the composition for $O(1)$ equality check (8 bytes: offset 64..72).
    pub comp_hash: u64,
    /// Start index in the DFA arena (4 bytes: offset 72..76).
    pub comp_offset: u32,
    /// Start index in the flattened-output arena (4 bytes: offset 76..80).
    pub flat_offset: u32,
    /// Number of valid transitions (1 byte: offset 80).
    pub trans_len: u8,
    /// Number of transformations in this state (1 byte: offset 81).
    pub comp_len: u8,
    /// Byte length of the flattened output in `flat_arena` (1 byte: offset 82).
    pub flat_len: u8,
    /// Explicit padding to 8-byte alignment (5 bytes: offset 83..88).
    pub _pad: [u8; 5],
}

const _: () = assert!(std::mem::size_of::<State>() == 88);

impl Default for State {
    fn default() -> Self {
        Self {
            bitset: [0; 2],
            trans_keys: [0; MAX_TRANS],
            trans_states: [0; MAX_TRANS],
            comp_hash: 0,
            comp_offset: 0,
            flat_offset: 0,
            trans_len: 0,
            comp_len: 0,
            flat_len: 0,
            _pad: [0; 5],
        }
    }
}

impl State {
    /// Performs a fast transition lookup on an ASCII key using bitset check + SWAR parallel search.
    ///
    /// The algorithm runs in constant time without branches for typical keys:
    /// 1. $O(1)$ rejection via the 128-bit bitset (1 bit per ASCII code).
    /// 2. If present, SWAR (SIMD Within A Register) compares 8 keys at once using 64-bit integers.
    ///
    /// # Arguments
    /// * `key` - The ASCII byte of the key to look up (0..127).
    ///
    /// # Returns
    /// The destination state ID (non-zero), or `0` if no transition exists.
    #[inline]
    pub const fn get_transition(&self, key: u8) -> u32 {
        // O(1) rejection: if the bit is not set, no transition exists.
        let idx = key as usize;
        if self.bitset[idx / 64] & (1u64 << (idx % 64)) == 0 {
            return 0;
        }

        let broadcast = (key as u64) * 0x0101010101010101;

        // Check chunk 0 (keys 0..8) using safe byte slice loading
        let c0 = u64::from_le_bytes([
            self.trans_keys[0],
            self.trans_keys[1],
            self.trans_keys[2],
            self.trans_keys[3],
            self.trans_keys[4],
            self.trans_keys[5],
            self.trans_keys[6],
            self.trans_keys[7],
        ]);
        let v0 = c0 ^ broadcast;
        let m0 = v0.wrapping_sub(0x0101010101010101) & !v0 & 0x8080808080808080;
        if m0 != 0 {
            let offset = (m0.trailing_zeros() / 8) as usize;
            if offset < self.trans_len as usize {
                return self.trans_states[offset] as u32;
            }
        }

        // Check chunk 1 (keys 8..16)
        if self.trans_len > 8 {
            let c1 = u64::from_le_bytes([
                self.trans_keys[8],
                self.trans_keys[9],
                self.trans_keys[10],
                self.trans_keys[11],
                self.trans_keys[12],
                self.trans_keys[13],
                self.trans_keys[14],
                self.trans_keys[15],
            ]);
            let v1 = c1 ^ broadcast;
            let m1 = v1.wrapping_sub(0x0101010101010101) & !v1 & 0x8080808080808080;
            if m1 != 0 {
                let offset = 8 + (m1.trailing_zeros() / 8) as usize;
                if offset < self.trans_len as usize {
                    return self.trans_states[offset] as u32;
                }
            }
        }

        0
    }

    /// Sets or updates a transition from this state on a given key.
    ///
    /// Updates both the 128-bit bitset and the key/state arrays.
    /// State IDs above [`MAX_STATE_ID`] are silently dropped (transition not stored).
    ///
    /// # Arguments
    /// * `key` - The ASCII key trigger.
    /// * `state_id` - The target DFA state ID.
    #[inline]
    pub fn set_transition(&mut self, key: u8, state_id: u32) {
        if state_id > MAX_STATE_ID {
            return;
        }
        let sid = state_id as u16;
        let idx = key as usize;
        self.bitset[idx / 64] |= 1u64 << (idx % 64);

        let len = self.trans_len as usize;
        for i in 0..len {
            if self.trans_keys[i] == key {
                self.trans_states[i] = sid;
                return;
            }
        }
        if len < MAX_TRANS {
            self.trans_keys[len] = key;
            self.trans_states[len] = sid;
            self.trans_len += 1;
        }
    }
}

/// The DFA core that manages states and transitions with Arena Allocation.
///
/// Stores all unique composition states and provides fast $O(1)$ lookups
/// without heap allocation during state transitions.
#[derive(Debug)]
pub struct Dfa {
    /// Array of all DFA states. State 0 is always the initial (empty) state.
    pub states: Vec<State>,
    /// Continuous arena storing transformation slices for all states.
    pub arena: Vec<Transformation>,
    /// Flattened UTF-8 output cache, keyed by `State::flat_offset` / `flat_len`.
    pub flat_arena: Vec<u8>,
    /// Maps composition hash to `state_id` for $O(1)$ deduplication and lookup.
    pub hash_to_state: FxHashMap<u64, u32>,
}

impl Clone for Dfa {
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            arena: self.arena.clone(),
            flat_arena: self.flat_arena.clone(),
            hash_to_state: self.hash_to_state.clone(),
        }
    }
}

impl Default for Dfa {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes a fast hash of a composition slice.
///
/// Uses the derived `Hash` impl, which hashes only the initialized fields and
/// never reads struct padding bytes. Hashing raw struct bytes directly would
/// read uninitialized padding (UB, flagged by Miri, and nondeterministic
/// across builds) — so the field-based hash is both correct and Miri-clean.
#[inline]
pub(crate) fn hash_composition(composition: &[Transformation]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = rustc_hash::FxHasher::default();
    composition.hash(&mut hasher);
    hasher.finish()
}

impl Dfa {
    /// Creates a new DFA with an initial empty state (state ID 0).
    ///
    /// Arenas start unallocated and grow on demand: a fresh `Engine` costs no
    /// ~20 KiB upfront, which matters for short-lived engines (batch items,
    /// FFI create/use/free, tests). Long sessions amortize the growth
    /// reallocations during warmup.
    pub fn new() -> Self {
        let mut dfa = Self {
            states: Vec::new(),
            arena: Vec::new(),
            flat_arena: Vec::new(),
            hash_to_state: FxHashMap::default(),
        };
        dfa.states.push(State::default());
        dfa.hash_to_state.insert(0, 0); // empty composition hash = 0
        dfa
    }

    /// Returns a reference to the [`State`] with the given ID.
    ///
    /// # Panics
    /// Panics if `id as usize` is out of bounds of the `states` vector.
    pub fn get_state(&self, id: u32) -> &State {
        &self.states[id as usize]
    }

    /// Retrieves the slice of [`Transformation`]s belonging to the specified state.
    pub fn get_composition(&self, state_id: u32) -> &[Transformation] {
        let state = &self.states[state_id as usize];
        let start = state.comp_offset as usize;
        let end = start + state.comp_len as usize;
        &self.arena[start..end]
    }

    /// Returns the cached flattened UTF-8 output for the specified state.
    ///
    /// Returns an empty slice if the state has no cached flatten (e.g. state 0).
    pub fn get_flat(&self, state_id: u32) -> &str {
        let state = &self.states[state_id as usize];
        let start = state.flat_offset as usize;
        let end = start + state.flat_len as usize;
        std::str::from_utf8(&self.flat_arena[start..end]).unwrap_or("")
    }

    /// Returns true when the DFA has hit [`DFA_MAX_STATES`] and is frozen.
    ///
    /// Frozen DFA still serves all cached transitions (fast path untouched);
    /// new compositions fall back to the rule-engine slow path.
    pub const fn is_full(&self) -> bool {
        self.states.len() >= DFA_MAX_STATES
    }

    /// Adds a new composition state to the DFA or returns the existing state ID if already present.
    ///
    /// Uses the precomputed hash and arena verification for fast collision-free deduplication.
    /// On hash collision, scans backwards through earlier states sharing the same
    /// `comp_hash` so previously inserted compositions remain reachable.
    /// Also caches the flattened lowercase output for zero-cost preedit reconstruction.
    ///
    /// When the DFA is full ([`DFA_MAX_STATES`]) or the next ID would exceed the
    /// `u16` transition address space, new compositions are **refused**: dedup
    /// hits still return their existing ID, but misses return `0` without
    /// allocating. Callers must skip `set_transition` on `0` and fall back to
    /// `find_state(...).unwrap_or(0)` (rule-engine slow path stays correct).
    pub fn add_state(&mut self, composition: &[Transformation]) -> u32 {
        // Arena-bounded: `comp_len`/`flat_len` below truncate into `u8`.
        debug_assert!(composition.len() <= MAX_TRANS);
        let hash = hash_composition(composition);

        // Fast path: hash match -> verify arena equality (no heap allocation).
        if let Some(&id) = self.hash_to_state.get(&hash) {
            if self.get_composition(id) == composition {
                return id;
            }
            // Hash collision: `hash_to_state` stores only the most recent state for
            // this hash. Scan backwards so older compositions with the same hash
            // are still found (and not duplicated).
            for old_id in (0..id).rev() {
                if self.states[old_id as usize].comp_hash == hash
                    && self.get_composition(old_id) == composition
                {
                    return old_id;
                }
            }
        }

        // Freeze: refuse new storage (also fixes the old zombie leak where
        // states kept pushing past u16 addressability with dropped edges).
        if self.states.len() >= DFA_MAX_STATES || self.states.len() > MAX_STATE_ID as usize {
            return 0;
        }

        let id = self.states.len() as u32;
        let comp_offset = self.arena.len() as u32;
        let comp_len = composition.len() as u8;

        // Cache the flattened lowercase output for P1 fast preedit reconstruction,
        // writing UTF-8 straight into the arena (no temporary `String`).
        let flat_offset = self.flat_arena.len() as u32;
        crate::flattener::append_flatten_bytes(
            composition,
            crate::mode::OutputOptions::LOWER_CASE,
            &mut self.flat_arena,
        );
        let flat_len = (self.flat_arena.len() - flat_offset as usize) as u8;

        self.arena.extend_from_slice(composition);
        self.states.push(State {
            comp_hash: hash,
            comp_offset,
            comp_len,
            flat_offset,
            flat_len,
            ..State::default()
        });

        self.hash_to_state.insert(hash, id);
        id
    }

    /// Returns the number of bytes stored in the flattened-output arena.
    pub const fn flat_len(&self) -> usize {
        self.flat_arena.len()
    }

    /// Returns Vec capacities (allocated slots) for memory accounting.
    pub const fn states_capacity(&self) -> usize {
        self.states.capacity()
    }

    /// Returns arena capacity in [`Transformation`] slots.
    pub const fn arena_capacity(&self) -> usize {
        self.arena.capacity()
    }

    /// Returns flat-arena capacity in bytes.
    pub const fn flat_capacity(&self) -> usize {
        self.flat_arena.capacity()
    }

    /// Returns the hash-map item capacity (≈7/8 of `SwissTable` buckets) for memory accounting.
    pub fn map_capacity(&self) -> usize {
        self.hash_to_state.capacity()
    }

    /// Estimates DFA heap usage (`used` bytes, without Vec slack).
    ///
    /// Formula: `states.len()*88 + arena.len()*16 + flat.len()*1 + map.len()*16`.
    /// Map per-entry cost uses 16 B (8 B key + 4 B value + alignment) + ~1 B
    /// `SwissTable` control byte, rounded to 16 B/entry for a conservative floor.
    /// Real allocator usage is slightly higher (control bytes + group overhead).
    pub fn memory_used_bytes(&self) -> usize {
        self.states.len() * std::mem::size_of::<State>()
            + self.arena.len() * std::mem::size_of::<Transformation>()
            + self.flat_arena.len()
            + self.hash_to_state.len() * 16
    }

    /// Estimates DFA heap allocation (`allocated` bytes, with Vec slack).
    ///
    /// Same formula as [`Self::memory_used_bytes`] but with capacities.
    ///
    /// This is a deliberate floor, not the allocator's real number:
    /// `capacity()` counts items (≈7/8 of `SwissTable` buckets), so the true
    /// map allocation is ~20% higher (`buckets × (16 B data + 1 B ctrl)`).
    /// Use it for relative before/after comparisons, not absolute RSS.
    pub fn memory_allocated_bytes(&self) -> usize {
        self.states.capacity() * std::mem::size_of::<State>()
            + self.arena.capacity() * std::mem::size_of::<Transformation>()
            + self.flat_arena.capacity()
            + self.hash_to_state.capacity() * 16
    }

    /// Checks whether `composition` equals the stored composition of `state_id`,
    /// ignoring `is_upper_case` bits.
    ///
    /// The DFA stores lowercase-canonical compositions (the engine lowercases
    /// before caching and re-applies case on fast-path restore), so an active
    /// buffer carrying uppercase bits still identifies the same state.
    /// Used to gate JIT edge linking: an edge may only be written when the
    /// source state identifies the pre-key composition.
    pub fn composition_matches_canonical(
        &self,
        state_id: u32,
        composition: &[Transformation],
    ) -> bool {
        let stored = self.get_composition(state_id);
        if stored.len() != composition.len() {
            return false;
        }
        stored.iter().zip(composition.iter()).all(|(a, b)| {
            a.key == b.key
                && a.effect_on == b.effect_on
                && a.result == b.result
                && a.target_raw() == b.target_raw()
                && a.effect == b.effect
                && a.effect_type == b.effect_type
        })
    }

    /// Finds the state ID corresponding to an existing composition slice, if present.
    ///
    /// On hash collision, falls back to a backward scan over earlier states with
    /// the same `comp_hash` (mirrors [`Dfa::add_state`]).
    pub fn find_state(&self, composition: &[Transformation]) -> Option<u32> {
        let hash = hash_composition(composition);
        let id = *self.hash_to_state.get(&hash)?;
        if self.get_composition(id) == composition {
            return Some(id);
        }
        // Hash collision: scan backwards for an earlier state with the same hash.
        (0..id).rev().find(|&old_id| {
            self.states[old_id as usize].comp_hash == hash
                && self.get_composition(old_id) == composition
        })
    }
}

/// A DFA compiler that pre-initializes common syllable states into a [`Dfa`].
pub struct DfaCompiler {
    /// The compiled DFA instance.
    pub dfa: Dfa,
    engine: crate::Engine,
}

impl DfaCompiler {
    /// Creates a new compiler instance for a given input method and configuration.
    pub fn new(im: &InputMethod, config: crate::Config) -> Self {
        let engine = crate::Engine::with_config(im.clone(), config);
        Self { dfa: Dfa::new(), engine }
    }

    /// Compiles common Vietnamese syllables into the DFA.
    pub fn compile_common(&mut self) {
        let fc = [
            "", "b", "c", "ch", "d", "dd", "g", "gh", "h", "k", "kh", "l", "m", "n", "nh", "ng",
            "ngh", "p", "ph", "q", "r", "s", "t", "th", "tr", "v", "x",
        ];
        let vowels = [
            "a", "e", "i", "o", "u", "y", "aa", "ee", "oo", "aw", "ow", "uw", "ai", "ao", "au",
            "ay", "ie", "oa", "oe", "oi", "ua", "ue", "ui", "uo", "uy",
        ];
        let tones = ["", "s", "f", "r", "x", "j"];

        // Stack-allocated buffer: max prefix "ngh" (3) + max vowel "uay" (3) + tone (1) = 7
        let mut buf = [0u8; 8];
        for &f in &fc {
            for &v in &vowels {
                for &t in &tones {
                    let mut pos = 0;
                    for &b in f.as_bytes() {
                        buf[pos] = b;
                        pos += 1;
                    }
                    for &b in v.as_bytes() {
                        buf[pos] = b;
                        pos += 1;
                    }
                    for &b in t.as_bytes() {
                        buf[pos] = b;
                        pos += 1;
                    }
                    if let Ok(seq) = std::str::from_utf8(&buf[..pos]) {
                        self.simulate_str(seq);
                    }
                }
            }
        }
    }

    fn simulate_str(&mut self, s: &str) {
        self.engine.reset();

        let mut current_state = 0u32;
        // State 0 identifies empty active text at word start. Afterwards the
        // previous state only identifies the pre-key composition when its own
        // add succeeded (a refusal leaves an unidentified residue that must
        // not gain word-start — or any — edges).
        let mut prev_known = true;
        for k in s.chars() {
            if !k.is_ascii() {
                continue;
            }

            let prev_state = current_state;
            let link_ok = prev_known;
            self.engine.process_key(k, crate::Mode::Vietnamese);

            let comp = self.engine.active_slice();
            current_state = self.dfa.add_state(comp);

            // Link the transition (skip when frozen: add_state returns 0 for
            // new compositions; 0 already means "no transition" on lookup).
            // Skip equally when the source state does not identify the
            // pre-key composition.
            if current_state != 0 {
                if link_ok {
                    self.dfa.states[prev_state as usize].set_transition(k as u8, current_state);
                }
                prev_known = true;
            } else {
                prev_known = comp.is_empty();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Transformation;
    use crate::input_method::EffectType;

    fn appending(ch: char) -> Transformation {
        Transformation::new(ch, ch, ch, None, 0, EffectType::Appending, false)
    }

    #[test]
    fn add_state_deduplicates_identical_composition() {
        let mut dfa = Dfa::new();
        let comp = [appending('a'), appending('b')];
        let id1 = dfa.add_state(&comp);
        let id2 = dfa.add_state(&comp);
        assert_eq!(id1, id2);
        assert_eq!(dfa.states.len(), 2); // state 0 (empty) + one real
    }

    #[test]
    fn find_state_roundtrip() {
        let mut dfa = Dfa::new();
        let comp_a = [appending('x')];
        let comp_b = [appending('y')];
        let id_a = dfa.add_state(&comp_a);
        let id_b = dfa.add_state(&comp_b);
        assert_ne!(id_a, id_b);
        assert_eq!(dfa.find_state(&comp_a), Some(id_a));
        assert_eq!(dfa.find_state(&comp_b), Some(id_b));
    }

    /// Regression: on hash collision, older compositions must remain reachable
    /// via `find_state` and `add_state` must not create duplicates.
    #[test]
    fn hash_collision_preserves_both_states() {
        let mut dfa = Dfa::new();
        let comp_a = [appending('p'), appending('q')];
        let comp_b = [appending('r'), appending('s')];

        let id_a = dfa.add_state(&comp_a);
        let id_b = dfa.add_state(&comp_b);
        assert_ne!(id_a, id_b);

        // Simulate a hash collision: force both states to share the same hash.
        let forced_hash = 0xDEAD_BEEF_CAFE_F00D_u64;
        dfa.states[id_a as usize].comp_hash = forced_hash;
        dfa.states[id_b as usize].comp_hash = forced_hash;
        dfa.hash_to_state.insert(forced_hash, id_b); // map points at newer state

        // Both compositions must still be findable.
        assert_eq!(dfa.find_state(&comp_a), Some(id_a));
        assert_eq!(dfa.find_state(&comp_b), Some(id_b));

        // Re-adding must not create duplicates.
        assert_eq!(dfa.add_state(&comp_a), id_a);
        assert_eq!(dfa.add_state(&comp_b), id_b);
        assert_eq!(dfa.states.len(), 3); // state 0 + id_a + id_b, no duplicate
    }

    #[test]
    fn freeze_bounds_memory_and_keeps_dedup() {
        let mut dfa = Dfa::new();
        // Fill to the cap with distinct 2-char compositions.
        // ('a'+hi, 'a'+lo) over i < 8192 gives fully distinct pairs.
        let mut i = 0u32;
        while !dfa.is_full() {
            let hi = char::from_u32('a' as u32 + (i / 64)).unwrap_or('x');
            let lo = char::from_u32('a' as u32 + (i % 64)).unwrap_or('y');
            let comp = [appending(hi), appending(lo)];
            let id = dfa.add_state(&comp);
            assert_ne!(id, 0, "distinct comp must allocate before full");
            i += 1;
            assert!(i < DFA_MAX_STATES as u32 + 10, "cap must trigger");
        }
        assert_eq!(dfa.states.len(), DFA_MAX_STATES);

        let used = dfa.memory_used_bytes();
        // 8192*88 states + arena + flat + map must stay in low single-digit MB.
        assert!(used < 4 * 1024 * 1024, "frozen DFA used={} too big", used);

        // A fresh composition is refused without allocating.
        // Char 0x10FFFF never appears in the fill range above.
        let fresh = [appending('\u{10FFFF}'), appending('\u{10FFFE}')];
        assert_eq!(dfa.find_state(&fresh), None);
        let before = dfa.states.len();
        assert_eq!(dfa.add_state(&fresh), 0);
        assert_eq!(dfa.states.len(), before);

        // Dedup hits still succeed when full.
        let dup = [appending('a'), appending('a')]; // i == 0
        let dup_id = dfa.add_state(&dup);
        assert_ne!(dup_id, 0);
        assert_eq!(dfa.states.len(), before);
        assert!(dfa.is_full());
    }

    #[test]
    fn memory_accounting_is_internally_consistent() {
        let mut dfa = Dfa::new();
        let comp = [appending('a'), appending('b')];
        dfa.add_state(&comp);
        assert!(dfa.memory_allocated_bytes() >= dfa.memory_used_bytes());
        assert!(dfa.states_capacity() >= dfa.states.len());
        assert!(dfa.arena_capacity() >= dfa.arena.len());
        assert!(dfa.flat_capacity() >= dfa.flat_len());
        assert!(dfa.map_capacity() >= dfa.hash_to_state.len());
    }

    #[test]
    fn composition_matches_canonical_ignores_case_only() {
        let mut dfa = Dfa::new();
        let lower = [appending('h'), appending('i')];
        let id = dfa.add_state(&lower);

        // Identical and case-only differences match (DFA is lowercase-canonical).
        assert!(dfa.composition_matches_canonical(id, &lower));
        let mut upper = lower;
        upper[0].is_upper_case = true;
        assert!(dfa.composition_matches_canonical(id, &upper));
        let mut all_upper = lower;
        for t in &mut all_upper {
            t.is_upper_case = true;
        }
        assert!(dfa.composition_matches_canonical(id, &all_upper));

        // Length, key, result, and target differences must not match.
        assert!(!dfa.composition_matches_canonical(id, &[]));
        assert!(!dfa.composition_matches_canonical(id, &[appending('h')]));
        let mut other_key = lower;
        other_key[1] = appending('o');
        assert!(!dfa.composition_matches_canonical(id, &other_key));
        let mut other_result = lower;
        other_result[1].result = 'î';
        assert!(!dfa.composition_matches_canonical(id, &other_result));
        let mut other_target = lower;
        other_target[1].set_target(Some(0));
        assert!(!dfa.composition_matches_canonical(id, &other_target));
    }
}
