#![deny(unused)]
#![deny(non_snake_case)]

mod accounts;
mod commands;
mod finance;
mod keys;
mod oracle;
mod state;
mod store;
mod token;
mod traits;
mod util;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(js_name = initializePanicHook)]
pub fn initialize_panic_hook() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
