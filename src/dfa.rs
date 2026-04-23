//! DFA-based engine for high-performance Vietnamese input method.

use crate::engine::Transformation;
use crate::input_method::InputMethod;
use std::collections::HashMap;

/// A DFA state representing a unique syllable composition.
#[derive(Clone, Debug)]
pub struct State {
    /// State transition table for 128 ASCII characters.
    pub transitions: [u32; 128],
    /// Start index in the DFA arena.
    pub comp_offset: u32,
    /// Number of transformations in this state.
    pub comp_len: u8,
}

impl Default for State {
    fn default() -> Self {
        Self { transitions: [0; 128], comp_offset: 0, comp_len: 0 }
    }
}

/// The DFA core that manages states and transitions with Arena Allocation.
pub struct Dfa {
    pub states: Vec<State>,
    pub arena: Vec<Transformation>,
    pub composition_to_state: HashMap<Box<[Transformation]>, u32>,
}

impl Default for Dfa {
    fn default() -> Self {
        Self::new()
    }
}

impl Dfa {
    /// Creates a new DFA with an initial empty state.
    pub fn new() -> Self {
        let mut dfa = Self {
            states: Vec::with_capacity(1024),
            arena: Vec::with_capacity(4096),
            composition_to_state: HashMap::new(),
        };
        let empty_comp: Box<[Transformation]> = Box::new([]);
        dfa.states.push(State::default());
        dfa.composition_to_state.insert(empty_comp, 0);
        dfa
    }

    pub fn get_state(&self, id: u32) -> &State {
        &self.states[id as usize]
    }

    pub fn get_composition(&self, state_id: u32) -> &[Transformation] {
        let state = &self.states[state_id as usize];
        let start = state.comp_offset as usize;
        let end = start + state.comp_len as usize;
        &self.arena[start..end]
    }

    pub fn add_state(&mut self, composition: &[Transformation]) -> u32 {
        if let Some(&id) = self.composition_to_state.get(composition) {
            return id;
        }

        let id = self.states.len() as u32;
        let comp_offset = self.arena.len() as u32;
        let comp_len = composition.len() as u8;

        self.arena.extend_from_slice(composition);
        self.states.push(State { transitions: [0; 128], comp_offset, comp_len });

        self.composition_to_state.insert(composition.to_vec().into_boxed_slice(), id);
        id
    }

    pub fn find_state(&self, composition: &[Transformation]) -> Option<u32> {
        self.composition_to_state.get(composition).copied()
    }
}

/// A DFA compiler that supports pre-initializing common states.
pub struct DfaCompiler<'a> {
    pub input_method: &'a InputMethod,
    pub flags: u32,
    pub dfa: Dfa,
}

impl<'a> DfaCompiler<'a> {
    pub fn new(im: &'a InputMethod, flags: u32) -> Self {
        Self { input_method: im, flags, dfa: Dfa::new() }
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

        for &f in &fc {
            for &v in &vowels {
                for &t in &tones {
                    let mut seq = String::with_capacity(8);
                    seq.push_str(f);
                    seq.push_str(v);
                    seq.push_str(t);
                    self.simulate_str(&seq);
                }
            }
        }
    }

    fn simulate_str(&mut self, s: &str) {
        let mut engine = crate::Engine::with_config(
            self.input_method.clone(),
            crate::Config::from_flags(self.flags),
        );

        let mut current_state = 0u32;
        for k in s.chars() {
            if !k.is_ascii() {
                continue;
            }

            let prev_state = current_state;
            engine.process_key(k, crate::Mode::Vietnamese);

            let comp = engine.active_slice();
            current_state = self.dfa.add_state(comp);

            // Link the transition
            self.dfa.states[prev_state as usize].transitions[k as usize] = current_state;
        }
    }
}
