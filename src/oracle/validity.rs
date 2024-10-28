#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum Validity {
    Fresh = 0,
    Stale = 1,
}
