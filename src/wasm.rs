#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct BambooWasmEngine {
    inner: crate::engine::Engine,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl BambooWasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::engine::Engine::new(
                crate::input_method::InputMethod::telex(),
            ),
        }
    }

    pub fn process_key(&mut self, key: char) -> String {
        self.inner.process_key(key, crate::mode::Mode::Vietnamese);
        self.inner.output()
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }

    pub fn output(&self) -> String {
        self.inner.output()
    }
}
