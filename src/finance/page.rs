use solana_program::clock::Clock;

use crate::util::revert;

use super::{Book, BookConfig, Decimal};

#[cfg(feature = "wasm")]
use {crate::util::Time, wasm_bindgen::prelude::*};

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Page {
    // Total amount borrowed or saved, including accrued interest.
    total: Decimal,
    // Total rewards received and not yet claimed.
    rewards: Decimal,
    // The most recent interest multiplier.
    multiplier: Decimal,
    // The most recent reward accumulator.
    accumulator: Decimal,
}

impl Page {
    pub const fn new() -> Self {
        Self {
            total: Decimal::zero(),
            rewards: Decimal::zero(),
            multiplier: Decimal::one(),
            accumulator: Decimal::zero(),
        }
    }

    pub const fn is_zero(&self) -> bool {
        self.total.is_zero()
    }

    fn accrue(&mut self, book: &mut Book, config: &BookConfig, clock: &Clock) {
        let (multiplier, accumulator) = book.get_multiplier_and_accumulator(config, clock);

        if multiplier != self.multiplier {
            if multiplier < self.multiplier {
                revert("Multiplier cannot decrease");
            }
            self.total *= multiplier / self.multiplier;
            self.multiplier = multiplier;
        }

        if accumulator != self.accumulator {
            if accumulator < self.accumulator {
                revert("Accumulator cannot decrease");
            }
            // accumulator = rewards accrued per principal
            self.rewards += (self.total / self.multiplier) * (accumulator - self.accumulator);
            self.accumulator = accumulator;
        }
    }
}

// public functions, should all have accrue as first statement
impl Page {
    pub fn get_total(&mut self, book: &mut Book, config: &BookConfig, clock: &Clock) -> Decimal {
        self.accrue(book, config, clock);
        self.total
    }

    pub fn claim_rewards(
        &mut self,
        book: &mut Book,
        config: &BookConfig,
        clock: &Clock,
    ) -> Decimal {
        self.accrue(book, config, clock);
        self.rewards.take()
    }

    pub fn add(&mut self, amount: Decimal, book: &mut Book, config: &BookConfig, clock: &Clock) {
        self.accrue(book, config, clock);
        self.total += amount;
        book.add(amount, config, clock);
    }

    pub fn subtract(
        &mut self,
        amount: Decimal,
        book: &mut Book,
        config: &BookConfig,
        clock: &Clock,
    ) {
        self.accrue(book, config, clock);
        if amount > self.total {
            revert("Insufficient balance");
        }
        self.total -= amount;
        book.subtract(amount, config, clock);
    }

    pub fn take(&mut self, book: &mut Book, config: &BookConfig, clock: &Clock) -> Decimal {
        self.accrue(book, config, clock);
        let total = self.total.take();
        book.subtract(total, config, clock);
        total
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
#[allow(non_snake_case)]
impl Page {
    #[wasm_bindgen(js_name = "projectTotal")]
    pub fn project_total_wasm(&self, book: &Book, config: &BookConfig, unixTimestamp: f64) -> f64 {
        let time = Time::from_unix_timestamp(unixTimestamp as u64);
        let multiplier = book.project_multiplier(config, time);
        (self.total * (multiplier / self.multiplier)).to_f64()
    }

    #[wasm_bindgen(js_name = "projectRewards")]
    #[allow(non_snake_case)]
    pub fn project_rewards_wasm(
        &self,
        book: &Book,
        config: &BookConfig,
        unixTimestamp: f64,
    ) -> f64 {
        let time = Time::from_unix_timestamp(unixTimestamp as u64);
        let accumulator = book.project_accumulator(config, time);
        (self.rewards
            + ((self.total / self.multiplier) * (accumulator.saturating_sub(self.accumulator))))
        .to_f64()
    }
}
