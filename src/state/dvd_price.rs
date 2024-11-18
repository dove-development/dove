use solana_program::clock::Clock;

use crate::{
    finance::{Decimal, InterestRate},
    util::Time,
};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Calculates the current price of DVD.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct DvdPrice {
    price: Decimal,
    last_updated: Time,
}

impl DvdPrice {
    pub const fn new(clock: &Clock) -> Self {
        Self {
            price: Decimal::one(),
            last_updated: Time::now(clock),
        }
    }
    fn accrue(&mut self, dvd_interest_rate: &InterestRate, clock: &Clock) {
        let secs_elapsed = self.last_updated.secs_elapsed(clock);
        if secs_elapsed == 0 {
            return;
        }

        self.price *= dvd_interest_rate.get_accumulation_factor(secs_elapsed);
        self.last_updated = Time::now(clock);
    }
    pub fn get(&mut self, dvd_interest_rate: &InterestRate, clock: &Clock) -> Decimal {
        self.accrue(dvd_interest_rate, clock);
        self.price
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl DvdPrice {
    #[wasm_bindgen(js_name = "projectPrice")]
    #[allow(non_snake_case)]
    pub fn project_price(&mut self, interestRate: &InterestRate, unixTimestamp: f64) -> Decimal {
        let secs_elapsed =
            Time::from_unix_timestamp(unixTimestamp as u64).secs_since(self.last_updated);
        self.price * interestRate.get_accumulation_factor(secs_elapsed)
    }
}
