use {
    crate::{
        accounts::Readonly,
        finance::{Decimal, InterestRate},
        oracle::{OracleKind, Pyth, Switchboard, UserFeed, Validity, ZeroFeed},
        state::DvdPrice,
        traits::Account,
        util::{require, Time},
    },
    solana_program::{clock::Clock, pubkey::Pubkey},
};

#[cfg(feature = "wasm")]
use {crate::util::b2pk, wasm_bindgen::prelude::*};

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Oracle {
    kind: OracleKind,
    key: Pubkey,
}

#[cfg(feature = "wasm")]
impl Oracle {
    pub const fn new(kind: OracleKind, key: Pubkey) -> Self {
        Self { kind, key }
    }
    pub const fn get_key(&self) -> Pubkey {
        self.key
    }
}

const STALE_AFTER_SECS_ELAPSED: u64 = 120;

impl Oracle {
    pub const fn zero() -> Self {
        Self {
            kind: OracleKind::ZeroFeed,
            key: Pubkey::new_from_array([0u8; 32]),
        }
    }

    fn query_usd_raw(
        &self,
        key: &Pubkey,
        data: &[u8],
        owner: &Pubkey,
        time: Time,
    ) -> Result<(Decimal, Validity), &'static str> {
        if key != &self.key {
            return Err("Oracle account mismatch");
        }
        let (price, price_time) = match self.kind {
            OracleKind::ZeroFeed => ZeroFeed::query(time),
            OracleKind::Pyth => Pyth::query(data, owner),
            OracleKind::Switchboard => Switchboard::query(data, owner),
            OracleKind::UserFeed => UserFeed::query(data, time),
        }?;
        let validity = match time.secs_since(price_time) {
            0..=STALE_AFTER_SECS_ELAPSED => Validity::Fresh,
            _ => Validity::Stale,
        };
        Ok((price, validity))
    }

    /// Returns the price, in DVD, of the oracle's asset.
    pub fn query_dvd(
        &self,
        oracle_account: Readonly,
        dvd_price: &mut DvdPrice,
        dvd_interest_rate: &InterestRate,
        clock: &Clock,
    ) -> Decimal {
        let key = oracle_account.get_info().key;
        let data = oracle_account.get_info().data.borrow();
        let owner = oracle_account.get_info().owner;
        let time = Time::now(clock);
        let (price, validity) = self.query_usd_raw(key, &data, owner, time).unwrap();
        require(validity == Validity::Fresh, "Oracle price is stale");
        price / dvd_price.get(dvd_interest_rate, clock)
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Oracle {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm(kind: OracleKind, key: Vec<u8>) -> Result<Oracle, String> {
        Ok(Self {
            kind,
            key: b2pk(&key)?,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn key(&self) -> Vec<u8> {
        self.key.to_bytes().to_vec()
    }

    #[wasm_bindgen(js_name = "zero")]
    pub fn zero_wasm() -> Self {
        Self::zero()
    }

    #[wasm_bindgen(js_name = getPriceNegativeIfStale)]
    #[allow(non_snake_case)]
    pub fn get_price_usd_negative_if_stale(
        &self,
        oracleKey: &[u8],
        oracleData: &[u8],
        oracleOwner: &[u8],
        unixTimestamp: f64,
    ) -> Result<f64, String> {
        let time = Time::from_unix_timestamp(unixTimestamp as u64);
        let oracle_key = b2pk(oracleKey)?;
        let oracle_owner = b2pk(oracleOwner)?;
        let (price, validity) = self
            .query_usd_raw(&oracle_key, oracleData, &oracle_owner, time)
            .map_err(|e| format!("Invalid collateral: {}", e))?;
        let price = price.to_f64();
        match validity {
            Validity::Fresh => Ok(price),
            Validity::Stale => Ok(-price),
        }
    }
}
