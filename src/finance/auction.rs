use crate::{finance::Decimal, traits::Pod, util::Time};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Configuration for a Dutch auction.
///
/// Example config: `begin_scale = 1.5`, `decay_rate = 0.9995`, `end_scale = 0.15`.
///
/// The auction is started `50%` above market price and decays by `0.05%` per second.
///
/// Market price is hit at `t = 810 seconds` or `13.5 minutes`.
///
/// The auction fails at `15%` of market price at `t = 4050 seconds` or `67.5 minutes`.
#[derive(Clone, Copy)]
#[repr(C)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct AuctionConfig {
    begin_scale: Decimal,
    decay_rate: Decimal,
    end_scale: Decimal,
}

unsafe impl Pod for AuctionConfig {}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AuctionConfig {
    #[wasm_bindgen(constructor)]
    #[allow(non_snake_case)]
    pub fn new(beginScale: f64, decayRate: f64, endScale: f64) -> Result<Self, String> {
        if beginScale <= endScale {
            return Err("begin_scale must be greater than end_scale".to_string());
        }
        if decayRate >= 1.0 {
            return Err("decay_rate must be less than 1".to_string());
        }
        if endScale >= 1.0 {
            return Err("end_scale must be less than 1".to_string());
        }
        Ok(Self {
            begin_scale: Decimal::from(beginScale),
            decay_rate: Decimal::from(decayRate),
            end_scale: Decimal::from(endScale),
        })
    }

    #[wasm_bindgen(js_name = "zero")]
    pub fn zero_wasm() -> Self {
        Self::zero()
    }

    #[wasm_bindgen(getter, js_name = "beginScale")]
    pub fn begin_scale(&self) -> f64 {
        self.begin_scale.to_f64()
    }

    #[wasm_bindgen(getter, js_name = "decayRate")]
    pub fn decay_rate(&self) -> f64 {
        self.decay_rate.to_f64()
    }

    #[wasm_bindgen(getter, js_name = "endScale")]
    pub fn end_scale(&self) -> f64 {
        self.end_scale.to_f64()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Auction<const N: usize> {
    market_prices: [Decimal; N],
    starting_time: Time,
}

impl<const N: usize> Auction<N> {
    pub fn new(market_prices: [Decimal; N], time: Time) -> Self {
        Self {
            market_prices,
            starting_time: time,
        }
    }

    fn scale(&self, config: &AuctionConfig, time: Time) -> Decimal {
        config.begin_scale * config.decay_rate.pow(time.secs_since(self.starting_time))
    }

    pub fn is_over(&self, config: &AuctionConfig, time: Time) -> bool {
        let scale = self.scale(config, time);
        scale <= config.end_scale
    }
    pub fn calculate_price(&self, config: &AuctionConfig, time: Time, index: usize) -> Decimal {
        let scale = self.scale(config, time);
        let market_price = *self
            .market_prices
            .get(index)
            .ok_or("Invalid index")
            .unwrap();
        market_price * scale
    }
    #[cfg(feature = "wasm")]
    pub fn get_secs_elapsed(&self, time: Time) -> u64 {
        time.secs_since(self.starting_time)
    }
    #[cfg(feature = "wasm")]
    pub fn get_fail_price(&self, config: &AuctionConfig, index: usize) -> Decimal {
        let market_price = *self
            .market_prices
            .get(index)
            .ok_or("Invalid index")
            .unwrap();
        market_price * config.end_scale
    }
}
