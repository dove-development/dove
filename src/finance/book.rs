use {
    crate::{
        finance::{Decimal, InterestRate, Schedule},
        util::{revert, Time, SECS_PER_DAY},
    },
    solana_program::clock::Clock,
};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Configuration for a Book
#[repr(C)]
#[derive(Clone, Copy, Default)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct BookConfig {
    interest_rate: InterestRate,
    reward_schedule: Schedule,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl BookConfig {
    #[wasm_bindgen(constructor)]
    #[allow(non_snake_case)]
    pub fn new(interestRate: InterestRate, rewardSchedule: Schedule) -> Self {
        Self {
            interest_rate: interestRate,
            reward_schedule: rewardSchedule,
        }
    }
    #[wasm_bindgen(getter, js_name = interestRate)]
    pub fn interest_rate(&self) -> InterestRate {
        self.interest_rate
    }
    #[wasm_bindgen(getter, js_name = rewardSchedule)]
    pub fn reward_schedule(&self) -> Schedule {
        self.reward_schedule
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Book {
    // Total amount borrowed or saved, including accrued interest.
    total: Decimal,
    // Total amount of rewards distributed.
    rewards: Decimal,
    // The interest multiplier, defined as the total interest accrued on 1 DVD deposited at the protocol's inception.
    multiplier: Decimal,
    // The reward accumulator, defined as the total rewards accrued on 1 DVD deposited at the protocol's inception.
    accumulator: Decimal,
    // The time at which this Book was created.
    creation_time: Time,
    // The time at which this Book was last updated.
    last_update: Time,
}

impl Book {
    pub const fn new(clock: &Clock) -> Self {
        let now = Time::now(clock);
        Self {
            total: Decimal::zero(),
            rewards: Decimal::zero(),
            multiplier: Decimal::one(),
            accumulator: Decimal::zero(),
            creation_time: now,
            last_update: now,
        }
    }

    fn project_total_and_multiplier(&self, config: &BookConfig, time: Time) -> (Decimal, Decimal) {
        let secs_elapsed = time.secs_since(self.last_update);
        let interest_factor = config.interest_rate.get_accumulation_factor(secs_elapsed);
        let new_total = self.total * interest_factor;
        let new_multiplier = self.multiplier * interest_factor;
        (new_total, new_multiplier)
    }

    fn project_rewards_and_accumulator(
        &self,
        config: &BookConfig,
        time: Time,
    ) -> (Decimal, Decimal) {
        let secs_since_last_update = time.secs_since(self.last_update);
        let secs_since_creation = time.secs_since(self.creation_time);

        let new_rewards = config.reward_schedule.integrate(
            Decimal::from(secs_since_creation - secs_since_last_update) / SECS_PER_DAY,
            Decimal::from(secs_since_creation) / SECS_PER_DAY,
        );
        let new_rewards_total = self.rewards + new_rewards;
        if self.total < Decimal::one() {
            // too small total, rewards are thrown into abyss
            return (self.rewards, self.accumulator);
        }
        // accumulator = rewards accrued per principal
        // accumulator = rewards / principal
        // accumulator = rewards / (total / multiplier)
        // accumulator = rewards * (multiplier / total)
        let new_accumulator = self.accumulator + ((new_rewards * self.multiplier) / self.total);
        (new_rewards_total, new_accumulator)
    }

    fn accrue(&mut self, config: &BookConfig, clock: &Clock) {
        let now = Time::now(clock);
        if now.secs_since(self.last_update) == 0 {
            return;
        }

        if !config.interest_rate.is_zero() {
            let (new_total, new_multiplier) = self.project_total_and_multiplier(config, now);
            self.total = new_total;
            self.multiplier = new_multiplier;
        }

        let (new_rewards, new_accumulator) = self.project_rewards_and_accumulator(config, now);
        self.rewards = new_rewards;
        self.accumulator = new_accumulator;
        self.last_update = now;
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
#[allow(non_snake_case)]
impl Book {
    #[wasm_bindgen(js_name = "projectTotal")]
    pub fn project_total(&self, config: &BookConfig, unixTimestamp: f64) -> f64 {
        let time = Time::from_unix_timestamp(unixTimestamp as u64);
        let (total, _) = self.project_total_and_multiplier(config, time);
        total.to_f64()
    }

    #[wasm_bindgen(js_name = "projectRewards")]
    #[allow(non_snake_case)]
    pub fn project_rewards(&self, config: &BookConfig, unixTimestamp: f64) -> f64 {
        let time = Time::from_unix_timestamp(unixTimestamp as u64);
        let (rewards, _) = self.project_rewards_and_accumulator(config, time);
        rewards.to_f64()
    }

    #[wasm_bindgen(getter, js_name = "creationTime")]
    pub fn creation_time(&self) -> f64 {
        self.creation_time.to_unix_timestamp() as f64
    }
}

#[cfg(feature = "wasm")]
impl Book {
    pub(super) fn project_multiplier(&self, config: &BookConfig, time: Time) -> Decimal {
        let (_, new_multiplier) = self.project_total_and_multiplier(config, time);
        new_multiplier
    }

    pub(super) fn project_accumulator(&self, config: &BookConfig, time: Time) -> Decimal {
        let (_, new_accumulator) = self.project_rewards_and_accumulator(config, time);
        new_accumulator
    }
}

// public functions, should all have accrue as first statement
impl Book {
    pub fn get_total(&mut self, config: &BookConfig, clock: &Clock) -> Decimal {
        self.accrue(config, clock);
        self.total
    }

    pub(super) fn get_multiplier_and_accumulator(
        &mut self,
        config: &BookConfig,
        clock: &Clock,
    ) -> (Decimal, Decimal) {
        self.accrue(config, clock);
        (self.multiplier, self.accumulator)
    }

    pub(super) fn add(&mut self, amount: Decimal, config: &BookConfig, clock: &Clock) {
        self.accrue(config, clock);
        self.total += amount;
    }

    pub(super) fn subtract(&mut self, amount: Decimal, config: &BookConfig, clock: &Clock) {
        self.accrue(config, clock);
        if amount > self.total {
            revert("Insufficient balance");
        }
        self.total -= amount;
    }
}
