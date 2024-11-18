use crate::finance::Decimal;

#[cfg(feature = "wasm")]
use {crate::util::SECS_PER_YEAR, wasm_bindgen::prelude::wasm_bindgen};

/// A continuously compounding interest rate.
#[repr(C)]
#[derive(Clone, Copy, Default)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct InterestRate {
    rate_per_sec: Decimal,
}

impl InterestRate {
    pub fn get_accumulation_factor(&self, secs_elapsed: u64) -> Decimal {
        (Decimal::one() + self.rate_per_sec).pow(secs_elapsed)
    }
    pub const fn is_zero(&self) -> bool {
        self.rate_per_sec.is_zero()
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl InterestRate {
    #[wasm_bindgen(constructor)]
    pub fn new(apy: f64) -> Result<Self, String> {
        if apy < 0.0 {
            return Err("APY must be non-negative".to_string());
        }
        if apy.is_nan() || apy.is_infinite() {
            return Err("APY must be a finite number".to_string());
        }
        Ok(Self {
            rate_per_sec: Decimal::from((1.0 + apy).ln()) / SECS_PER_YEAR,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn apy(&self) -> f64 {
        (self.rate_per_sec * SECS_PER_YEAR).to_f64().exp() - 1.0
    }

    #[wasm_bindgen(getter)]
    pub fn zero() -> Self {
        Self {
            rate_per_sec: Decimal::zero(),
        }
    }
}
