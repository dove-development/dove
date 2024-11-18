use crate::{
    finance::{AuctionConfig, BookConfig, Decimal, InterestRate},
    oracle::Oracle,
    state::{OfferingConfig, SovereignAuth},
    store::VaultConfig,
    traits::Pod,
};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use super::flash_mint::FlashMintConfig;

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Config {
    max_ltv: Decimal,
    dvd_interest_rate: InterestRate,
    dove_oracle: Oracle,
    auction_config: AuctionConfig,
    debt_config: BookConfig,
    flash_mint_config: FlashMintConfig,
    offering_config: OfferingConfig,
    savings_config: BookConfig,
    vault_config: VaultConfig,
}

impl Config {
    pub const fn get_max_ltv(&self) -> Decimal {
        self.max_ltv
    }

    pub const fn get_dvd_interest_rate(&self) -> &InterestRate {
        &self.dvd_interest_rate
    }

    pub const fn get_dove_oracle(&self) -> &Oracle {
        &self.dove_oracle
    }

    pub const fn get_auction_config(&self) -> &AuctionConfig {
        &self.auction_config
    }

    pub const fn get_debt_config(&self) -> &BookConfig {
        &self.debt_config
    }

    pub const fn get_flash_mint_config(&self) -> &FlashMintConfig {
        &self.flash_mint_config
    }

    pub const fn get_offering_config(&self) -> &OfferingConfig {
        &self.offering_config
    }

    pub const fn get_savings_config(&self) -> &BookConfig {
        &self.savings_config
    }

    pub const fn get_vault_config(&self) -> &VaultConfig {
        &self.vault_config
    }

    pub fn update(&mut self, _: SovereignAuth, new_config: Config) {
        *self = new_config;
    }
}
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Config {
    #[wasm_bindgen(constructor)]
    #[allow(non_snake_case)]
    pub fn new(
        maxLtv: f64,
        dvdInterestRate: InterestRate,
        doveOracle: Oracle,
        auctionConfig: AuctionConfig,
        debtConfig: BookConfig,
        flashMintConfig: FlashMintConfig,
        offeringConfig: OfferingConfig,
        savingsConfig: BookConfig,
        vaultConfig: VaultConfig,
    ) -> Result<Self, String> {
        if maxLtv <= 0.0 || maxLtv >= 1.0 {
            return Err("max_ltv must be between 0 and 1".to_string());
        }
        Ok(Self {
            max_ltv: Decimal::from(maxLtv),
            dvd_interest_rate: dvdInterestRate,
            dove_oracle: doveOracle,
            auction_config: auctionConfig,
            debt_config: debtConfig,
            flash_mint_config: flashMintConfig,
            offering_config: offeringConfig,
            savings_config: savingsConfig,
            vault_config: vaultConfig,
        })
    }

    #[wasm_bindgen(getter, js_name = "maxLtv")]
    pub fn max_ltv(&self) -> f64 {
        self.max_ltv.to_f64()
    }

    #[wasm_bindgen(getter, js_name = "dvdInterestRate")]
    pub fn dvd_interest_rate(&self) -> InterestRate {
        self.dvd_interest_rate
    }

    #[wasm_bindgen(getter, js_name = "doveOracle")]
    pub fn dove_oracle(&self) -> Oracle {
        self.dove_oracle
    }

    #[wasm_bindgen(getter, js_name = "auctionConfig")]
    pub fn auction_config(&self) -> AuctionConfig {
        self.auction_config
    }

    #[wasm_bindgen(getter, js_name = "debtConfig")]
    pub fn debt_config(&self) -> BookConfig {
        self.debt_config
    }

    #[wasm_bindgen(getter, js_name = "flashMintConfig")]
    pub fn flash_mint_config(&self) -> FlashMintConfig {
        self.flash_mint_config
    }

    #[wasm_bindgen(getter, js_name = "offeringConfig")]
    pub fn offering_config(&self) -> OfferingConfig {
        self.offering_config
    }

    #[wasm_bindgen(getter, js_name = "savingsConfig")]
    pub fn savings_config(&self) -> BookConfig {
        self.savings_config
    }

    #[wasm_bindgen(getter, js_name = "vaultConfig")]
    pub fn vault_config(&self) -> VaultConfig {
        self.vault_config
    }
}

unsafe impl Pod for Config {
    const NAME: &'static str = "Config";
}
