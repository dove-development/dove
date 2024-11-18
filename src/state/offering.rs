use crate::{
    accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
    finance::{Auction, AuctionConfig, Book, BookConfig, Decimal, InterestRate},
    oracle::Oracle,
    state::{DvdPrice, StableDvd},
    store::Authority,
    token::Token,
    traits::Pod,
    util::{revert, Time},
};
use solana_program::clock::Clock;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Configuration for the offering system.
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct OfferingConfig {
    /// The upper limit, in DVD, for the system surplus.
    /// Beyond this, DVD is minted and offered for DOVE, which is burned.
    surplus_limit: Decimal,
    /// The upper limit, in DVD, for the system deficit.
    /// Beyond this, DOVE is minted and offered for DVD, which is burned.
    deficit_limit: Decimal,
    /// The amount of DVD to offer at once during DVD offerings.
    dvd_offering_size: Decimal,
    /// The amount of DOVE to offer at once during DOVE offerings.
    dove_offering_size: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl OfferingConfig {
    #[wasm_bindgen(constructor)]
    #[allow(non_snake_case)]
    pub fn new(
        surplusLimit: f64,
        deficitLimit: f64,
        dvdOfferingSize: f64,
        doveOfferingSize: f64,
    ) -> Self {
        Self {
            surplus_limit: Decimal::from(surplusLimit),
            deficit_limit: Decimal::from(deficitLimit),
            dvd_offering_size: Decimal::from(dvdOfferingSize),
            dove_offering_size: Decimal::from(doveOfferingSize),
        }
    }

    #[wasm_bindgen(getter, js_name = surplusLimit)]
    pub fn surplus_limit(&self) -> f64 {
        self.surplus_limit.to_f64()
    }

    #[wasm_bindgen(getter, js_name = deficitLimit)]
    pub fn deficit_limit(&self) -> f64 {
        self.deficit_limit.to_f64()
    }

    #[wasm_bindgen(getter, js_name = dvdOfferingSize)]
    pub fn dvd_offering_size(&self) -> f64 {
        self.dvd_offering_size.to_f64()
    }

    #[wasm_bindgen(getter, js_name = doveOfferingSize)]
    pub fn dove_offering_size(&self) -> f64 {
        self.dove_offering_size.to_f64()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum OfferingState {
    Inactive,
    DoveOffering {
        qty_remaining: Decimal,
        auction: Auction<1>,
    },
    DvdOffering {
        qty_remaining: Decimal,
        auction: Auction<1>,
    },
}

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Offering {
    state: OfferingState,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
#[allow(non_snake_case)]
impl Offering {
    #[wasm_bindgen(getter, js_name = amount)]
    pub fn amount(&self) -> f64 {
        match self.state {
            OfferingState::DvdOffering { qty_remaining, .. } => qty_remaining.to_f64(),
            OfferingState::DoveOffering { qty_remaining, .. } => qty_remaining.to_f64(),
            OfferingState::Inactive => 0.0,
        }
    }

    #[wasm_bindgen(js_name = getPrice)]
    #[allow(non_snake_case)]
    pub fn get_price(&self, config: &AuctionConfig, unixTimestamp: f64) -> f64 {
        let time = Time::from_unix_timestamp(unixTimestamp as u64);
        match self.state {
            OfferingState::DvdOffering { auction, .. } => {
                auction.calculate_price(config, time, 0).to_f64()
            }
            OfferingState::DoveOffering { auction, .. } => {
                auction.calculate_price(config, time, 0).to_f64()
            }
            OfferingState::Inactive => 0.0,
        }
    }

    #[wasm_bindgen(getter, js_name = isActive)]
    pub fn is_active(&self) -> bool {
        match self.state {
            OfferingState::DvdOffering { .. } => true,
            OfferingState::DoveOffering { .. } => true,
            OfferingState::Inactive => false,
        }
    }

    #[wasm_bindgen(js_name = getSecsElapsed)]
    #[allow(non_snake_case)]
    pub fn get_secs_elapsed(&self, unixTimestamp: f64) -> u64 {
        let time = Time::from_unix_timestamp(unixTimestamp as u64);
        match self.state {
            OfferingState::DvdOffering { auction, .. } => auction.get_secs_elapsed(time),
            OfferingState::DoveOffering { auction, .. } => auction.get_secs_elapsed(time),
            OfferingState::Inactive => 0,
        }
    }

    #[wasm_bindgen(js_name = getFailPrice)]
    pub fn get_fail_price(&self, config: &AuctionConfig) -> f64 {
        match self.state {
            OfferingState::DvdOffering { auction, .. } => {
                auction.get_fail_price(config, 0).to_f64()
            }
            OfferingState::DoveOffering { auction, .. } => {
                auction.get_fail_price(config, 0).to_f64()
            }
            OfferingState::Inactive => 0.0,
        }
    }

    #[wasm_bindgen(getter, js_name = isDvd)]
    pub fn is_dvd(&self) -> bool {
        match self.state {
            OfferingState::DvdOffering { .. } => true,
            _ => false,
        }
    }
}

impl Offering {
    pub fn new() -> Self {
        Self {
            state: OfferingState::Inactive,
        }
    }

    pub fn start(
        &mut self,
        clock: &Clock,
        oracle_account: Readonly,
        debt: &mut Book,
        savings: &mut Book,
        dvd: &mut Token,
        dvd_price: &mut DvdPrice,
        dvd_interest_rate: &InterestRate,
        stable_dvd: &mut StableDvd,
        dove_oracle: &Oracle,
        offering_config: &OfferingConfig,
        debt_config: &BookConfig,
        savings_config: &BookConfig,
    ) {
        match self.state {
            OfferingState::Inactive => (),
            _ => revert("can't start new debt/equity offering until current is finished"),
        }
        let dove_price = dove_oracle.query_dvd(oracle_account, dvd_price, dvd_interest_rate, clock);

        let assets = debt.get_total(debt_config, clock) + stable_dvd.get_circulating();
        let liabilities = dvd.get_supply() + savings.get_total(savings_config, clock);
        if assets > liabilities {
            let surplus = assets - liabilities;
            if surplus <= offering_config.surplus_limit {
                revert("surplus is too low to merit auction");
            }
            let debt_price = Decimal::one() / dove_price;
            self.state = OfferingState::DvdOffering {
                qty_remaining: offering_config.dvd_offering_size,
                auction: Auction::new([debt_price], Time::now(clock)),
            }
        } else {
            let deficit = liabilities - assets;
            if deficit <= offering_config.deficit_limit {
                revert("deficit is too low to merit auction");
            }
            self.state = OfferingState::DoveOffering {
                qty_remaining: offering_config.dove_offering_size,
                auction: Auction::new([dove_price], Time::now(clock)),
            };
        }
    }

    pub fn end(&mut self, clock: &Clock, auction_config: &AuctionConfig) {
        match &self.state {
            OfferingState::Inactive => revert("No active offering to end"),
            OfferingState::DvdOffering {
                auction,
                qty_remaining,
            }
            | OfferingState::DoveOffering {
                auction,
                qty_remaining,
            } => {
                if !qty_remaining.is_zero() && !auction.is_over(auction_config, Time::now(clock)) {
                    revert("Auction has not ended yet");
                }
                self.state = OfferingState::Inactive;
            }
        }
    }

    pub fn buy(
        &mut self,
        requested_base_amount: Decimal,
        clock: &Clock,
        dvd: &mut Token,
        dove: &mut Token,
        authority: Authority,
        auction_config: &AuctionConfig,
        user_account: Signer,
        dvd_mint_account: MintAccount<Writable>,
        dvd_account: TokenAccount<Writable>,
        dove_mint_account: MintAccount<Writable>,
        dove_token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
    ) {
        match &mut self.state {
            OfferingState::DvdOffering {
                qty_remaining,
                auction,
            } => {
                let dvd_price = auction.calculate_price(auction_config, Time::now(clock), 0);
                let dvd_amount = (requested_base_amount / dvd_price).min(*qty_remaining);
                if dvd_amount.is_zero() {
                    revert("nothing to buy");
                }
                let dove_amount = (dvd_amount * dvd_price).min(requested_base_amount);

                dvd.mint(
                    dvd_amount,
                    dvd_mint_account,
                    dvd_account,
                    authority,
                    token_program_account,
                );

                dove.burn(
                    dove_amount,
                    dove_mint_account,
                    dove_token_account,
                    token_program_account,
                    user_account,
                );

                *qty_remaining -= dvd_amount;
            }
            OfferingState::DoveOffering {
                qty_remaining,
                auction,
            } => {
                let dove_price = auction.calculate_price(auction_config, Time::now(clock), 0);
                let dove_amount = (requested_base_amount / dove_price).min(*qty_remaining);
                if dove_amount.is_zero() {
                    revert("nothing to buy");
                }
                let dvd_amount = (dove_amount * dove_price).min(requested_base_amount);

                dove.mint(
                    dove_amount,
                    dove_mint_account,
                    dove_token_account,
                    authority,
                    token_program_account,
                );

                dvd.burn(
                    dvd_amount,
                    dvd_mint_account,
                    dvd_account,
                    token_program_account,
                    user_account,
                );

                *qty_remaining -= dove_amount;
            }
            OfferingState::Inactive => revert("No active offering"),
        }
    }
}

unsafe impl Pod for Offering {
    const NAME: &'static str = "Offering";
}
