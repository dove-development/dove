use crate::{
    accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
    finance::{Decimal, Schedule},
    store::Authority,
    token::Token,
    traits::{Account, Pod},
    util::{require, Time, SECS_PER_DAY},
};
use solana_program::{clock::Clock, pubkey::Pubkey};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
#[repr(C)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Vesting {
    recipient: Pubkey,
    schedule: Schedule,
    start_time: Time,
    last_updated_time: Time,
}

unsafe impl Pod for Vesting {
    const NAME: &'static str = "Vesting";
}

impl Vesting {
    pub fn new(clock: &Clock, recipient: Pubkey, schedule: Schedule) -> Self {
        let now = Time::now(clock);
        Self {
            recipient,
            start_time: now,
            last_updated_time: now,
            schedule,
        }
    }

    pub fn claim_emission(
        &mut self,
        user: Signer,
        dove: &mut Token,
        dove_mint_account: MintAccount<Writable>,
        dove_token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
        authority: Authority,
        clock: &Clock,
    ) {
        require(
            &self.recipient == user.get_info().key,
            "Vesting::claim_emission: not authorized",
        );
        let secs_since_creation = self.start_time.secs_elapsed(clock);
        let secs_since_last_update = self.last_updated_time.secs_elapsed(clock);
        if secs_since_last_update == 0 {
            return;
        }

        let emission_due = self.schedule.integrate(
            Decimal::from(secs_since_creation - secs_since_last_update)
                / Decimal::from(SECS_PER_DAY),
            Decimal::from(secs_since_creation) / Decimal::from(SECS_PER_DAY),
        );
        dove.mint(
            emission_due,
            dove_mint_account,
            dove_token_account,
            authority,
            token_program_account,
        );
        self.last_updated_time = Time::now(clock);
    }

    pub fn update_recipient(&mut self, user: &Signer, new_recipient: Readonly) {
        require(
            &self.recipient == user.get_info().key,
            "Vesting::update_recipient: not authorized",
        );
        self.recipient = *new_recipient.get_info().key;
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Vesting {
    #[wasm_bindgen(getter)]
    pub fn recipient(&self) -> Vec<u8> {
        self.recipient.to_bytes().to_vec()
    }

    #[wasm_bindgen(getter)]
    pub fn schedule(&self) -> Schedule {
        self.schedule
    }

    #[wasm_bindgen(js_name = getEmissionDue)]
    #[allow(non_snake_case)]
    pub fn get_emission_due(&self, unixTimestamp: f64) -> f64 {
        let time = Time::from_unix_timestamp(unixTimestamp as u64);
        let secs_since_creation = time.secs_since(self.start_time);
        let secs_since_last_update = time.secs_since(self.last_updated_time);

        if secs_since_last_update == 0 {
            return 0.0;
        }

        let emission_due = self.schedule.integrate(
            Decimal::from(secs_since_creation - secs_since_last_update)
                / Decimal::from(SECS_PER_DAY),
            Decimal::from(secs_since_creation) / Decimal::from(SECS_PER_DAY),
        );

        emission_due.to_f64()
    }

    #[wasm_bindgen(getter)]
    pub fn distributed(&self) -> f64 {
        let secs_since_creation = self.last_updated_time.secs_since(self.start_time);
        let days_since_creation = Decimal::from(secs_since_creation) / Decimal::from(SECS_PER_DAY);

        let distributed = self
            .schedule
            .integrate(Decimal::from(0), days_since_creation);

        distributed.to_f64()
    }
}
