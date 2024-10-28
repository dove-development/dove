use crate::finance::Decimal;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// The total number of DVD minted with Stability modules.
/// This is just a tracking number, and should be kept in sync with
/// (but does not affect) the actual circulating supply.
#[derive(Clone, Copy)]
#[repr(transparent)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct StableDvd {
    circulating: Decimal
}

impl StableDvd {
    pub const fn new() -> Self {
        Self { circulating: Decimal::zero()}
    }

    pub fn increase(&mut self, amount: Decimal) {
        self.circulating += amount;
    }

    pub fn decrease(&mut self, amount: Decimal) {
        self.circulating -= amount;
    }

    pub const fn get_circulating(&self) -> Decimal {
        self.circulating
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl StableDvd {
    #[wasm_bindgen(getter)]
    pub fn circulating(&self) -> f64 {
        self.circulating.to_f64()
    }
}
