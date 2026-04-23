//! DFA-based engine for high-performance Vietnamese input method.

use crate::engine::Transformation;
use crate::input_method::InputMethod;
use std::collections::HashMap;

/// A DFA state representing a unique syllable composition.
///
/// Each state stores transitions to next states based on ASCII keys
/// and the corresponding transformation stack (composition).
#[derive(Clone, Debug)]
pub struct State {
    /// State transition table for 128 ASCII characters.
    /// A value of 0 indicates no transition or fallback to the Rule Engine.
    pub transitions: [u32; 128],
    /// The transformation stack that makes up this state.
    pub composition: Box<[Transformation]>,
}

impl Default for State {
    fn default() -> Self {
        Self { transitions: [0; 128], composition: Box::new([]) }
    }
}

/// The DFA (Deterministic Finite Automaton) core that manages states and transitions.
///
/// This DFA supports a Lazy JIT mechanism, allowing the Engine to learn new states
/// during execution to optimize performance.
pub struct Dfa {
    /// List of states in the DFA.
    pub states: Vec<State>,
    /// Hash map mapping transformation stacks to state IDs to avoid duplicates.
    pub composition_to_state: HashMap<Box<[Transformation]>, u32>,
}

impl Default for Dfa {
    fn default() -> Self {
        Self::new()
    }
}

impl Dfa {
    /// Creates a new DFA with an initial empty state (empty syllable).
    pub fn new() -> Self {
        let mut dfa =
            Self { states: Vec::with_capacity(1024), composition_to_state: HashMap::new() };
        // Initial empty state
        let empty_comp = Box::new([]);
        dfa.states.push(State::default());
        dfa.composition_to_state.insert(empty_comp, 0);
        dfa
    }

    /// Retrieves a reference to a state by its ID.
    pub fn get_state(&self, id: u32) -> &State {
        &self.states[id as usize]
    }

    /// Adds a new state to the DFA from a transformation stack.
    /// If the state already exists, it returns the existing state ID.
    pub fn add_state(&mut self, composition: &[Transformation]) -> u32 {
        let comp_box: Box<[Transformation]> = composition.to_vec().into_boxed_slice();
        if let Some(&id) = self.composition_to_state.get(&comp_box) {
            return id;
        }

        let id = self.states.len() as u32;
        self.states.push(State { transitions: [0; 128], composition: comp_box.clone() });
        self.composition_to_state.insert(comp_box, id);
        id
    }

    /// Finds the state ID corresponding to a transformation stack.
    pub fn find_state(&self, composition: &[Transformation]) -> Option<u32> {
        self.composition_to_state.get(composition).copied()
    }
}

/// A DFA compiler that supports pre-initializing common states.
#[allow(dead_code)]
pub struct DfaCompiler<'a> {
    pub input_method: &'a InputMethod,
    pub flags: u32,
    pub composition_to_state: HashMap<Vec<Transformation>, u32>,
    pub dfa: Dfa,
}

#[allow(dead_code)]
impl<'a> DfaCompiler<'a> {
    pub fn new(im: &'a InputMethod, flags: u32) -> Self {
        let mut compiler =
            Self { input_method: im, flags, composition_to_state: HashMap::new(), dfa: Dfa::new() };
        compiler.composition_to_state.insert(Vec::new(), 0);
        compiler
    }

    /// Compiles common Vietnamese syllables into the DFA to reduce initial latency.
    pub fn compile_common(&mut self) {
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
        let tones = ['s', 'f', 'r', 'x', 'j']; // Telex tone keys

        // Compile single vowels and basic tone combinations
        for &v in &vowels {
            self.simulate_key(v);
            for &t in &tones {
                self.simulate_sequence(&[v, t]);
            }
        }

        // Common double vowel combinations
        let double_vowels = ["aa", "ee", "oo", "aw", "ow", "uw"];
        for s in double_vowels {
            self.simulate_str(s);
        }
    }

    fn simulate_key(&mut self, key: char) -> u32 {
        let mut engine = crate::Engine::with_config(
            self.input_method.clone(),
            crate::Config::from_flags(self.flags),
        );
        engine.process_key(key, crate::Mode::Vietnamese);
        self.dfa.add_state(engine.active_slice())
    }

    fn simulate_sequence(&mut self, keys: &[char]) -> u32 {
        let mut engine = crate::Engine::with_config(
            self.input_method.clone(),
            crate::Config::from_flags(self.flags),
        );
        for &k in keys {
            engine.process_key(k, crate::Mode::Vietnamese);
        }
        self.dfa.add_state(engine.active_slice())
    }

    fn simulate_str(&mut self, s: &str) {
        let mut engine = crate::Engine::with_config(
            self.input_method.clone(),
            crate::Config::from_flags(self.flags),
        );
        for k in s.chars() {
            engine.process_key(k, crate::Mode::Vietnamese);
        }
        self.dfa.add_state(engine.active_slice());
    }
}
