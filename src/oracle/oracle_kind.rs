#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum OracleKind {
    ZeroFeed = 0,
    Pyth = 1,
    Switchboard = 2,
    UserFeed = 3,
}
