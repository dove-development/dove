use crate::{
    finance::{AuctionConfig, Book, BookConfig},
    token::Token,
    util::{revert, Time},
};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use {
    crate::{
        accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::{Auction, Decimal, Page},
        store::{Authority, Collateral},
        token::Reserve,
        traits::{Account, Pod, Store, StoreAuth},
        util::{require, List},
    },
    solana_program::{clock::Clock, pubkey::Pubkey},
};

const MAX_RESERVES: usize = 6;

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VaultConfig {
    // The percentage of the loan amount to be added to the vault debt as a penalty during liquidation.
    liquidation_penalty_rate: Decimal,
    // The maximum liquidation reward in DVD.
    liquidation_reward_cap: Decimal,
    // The percentage of the loan amount that is rewarded to the caller of liquidate().
    // This does not actually liquidate any collateral, but places the Vault up for auction.
    // The actual reward is the minimum of this percentage and the liquidation_reward_cap.
    liquidation_reward_rate: Decimal,
    // The maximum reward for marking an auction as failed.
    auction_failure_reward_cap: Decimal,
    // The percentage of the still outstanding loan amount that is rewarded for marking an auction as failed.
    // The actual reward is the minimum of this percentage and the auction_failure_reward_cap.
    auction_failure_reward_rate: Decimal,
}

unsafe impl Pod for VaultConfig {
    const NAME: &'static str = "VaultConfig";
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VaultConfig {
    #[wasm_bindgen(constructor)]
    #[allow(non_snake_case)]
    pub fn new(
        liquidationPenaltyRate: f64,
        liquidationRewardCap: f64,
        liquidationRewardRate: f64,
        auctionFailureRewardCap: f64,
        auctionFailureRewardRate: f64,
    ) -> Self {
        Self {
            liquidation_penalty_rate: Decimal::from(liquidationPenaltyRate),
            liquidation_reward_cap: Decimal::from(liquidationRewardCap),
            liquidation_reward_rate: Decimal::from(liquidationRewardRate),
            auction_failure_reward_cap: Decimal::from(auctionFailureRewardCap),
            auction_failure_reward_rate: Decimal::from(auctionFailureRewardRate),
        }
    }

    #[wasm_bindgen(js_name = "zero")]
    pub fn zero_wasm() -> Self {
        Self::zero()
    }

    #[wasm_bindgen(getter, js_name = "liquidationRewardCap")]
    pub fn liquidation_reward_cap(&self) -> f64 {
        self.liquidation_reward_cap.to_f64()
    }

    #[wasm_bindgen(getter, js_name = "liquidationRewardRate")]
    pub fn liquidation_reward_rate(&self) -> f64 {
        self.liquidation_reward_rate.to_f64()
    }

    #[wasm_bindgen(getter, js_name = "auctionFailureRewardCap")]
    pub fn auction_failure_reward_cap(&self) -> f64 {
        self.auction_failure_reward_cap.to_f64()
    }

    #[wasm_bindgen(getter, js_name = "auctionFailureRewardRate")]
    pub fn auction_failure_reward_rate(&self) -> f64 {
        self.auction_failure_reward_rate.to_f64()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Vault {
    initialized: bool,
    nonce: u8,
    owner: Pubkey,
    debt: Page,
    reserves: List<Reserve, MAX_RESERVES>,
    auction: Option<Auction<MAX_RESERVES>>,
}

impl Store for Vault {
    const SEED_PREFIX: &'static str = "vault";
    type Params = Signer;
    type DeriveData<'a> = &'a Pubkey;
    type CreateData<'a> = Signer;
    type LoadData = ();
    type LoadAuthData = Signer;

    fn get_seeds_on_derive<'a>(user_account: Self::DeriveData<'a>) -> [&'a [u8]; 2] {
        [user_account.as_bytes(), &[]]
    }
    fn get_seeds_on_create<'a>(user_account: Signer) -> [&'a [u8]; 2] {
        [user_account.get_info().key.as_bytes(), &[]]
    }
    fn get_seeds_on_load(&self, _: ()) -> [&[u8]; 2] {
        [self.owner.as_ref(), &[]]
    }
    fn get_seeds_on_load_auth(&self, user_account: Signer) -> [&'static [u8]; 2] {
        [user_account.get_info().key.as_bytes(), &[]]
    }

    fn initialize(&mut self, nonce: u8, user_account: Signer) {
        self.initialized = true;
        self.nonce = nonce;
        self.owner = *user_account.get_info().key;
        self.debt = Page::new();
        self.reserves = List::new();
        self.auction = None;
    }
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    fn get_nonce(&self) -> u8 {
        self.nonce
    }
}

// Authorized functions
impl Vault {
    pub fn create_reserve(&mut self, auth: StoreAuth<Self>, collateral: &Collateral) {
        _ = auth;
        require(self.auction.is_none(), "Vault is liquidated");
        require(
            !self
                .reserves
                .iter()
                .any(|r| r.get_mint() == collateral.get_mint()),
            "Reserve already exists",
        );
        let reserve = Reserve::new(collateral);
        self.reserves.push(reserve);
    }

    pub fn remove_reserve(&mut self, auth: StoreAuth<Self>, collateral_mint: &Pubkey) {
        _ = auth;
        require(self.auction.is_none(), "Vault is liquidated");

        let reserve_index = self
            .reserves
            .iter()
            .position(|r| r.get_mint().get_key() == collateral_mint)
            .ok_or("Reserve not found")
            .unwrap();

        let reserve = &self.reserves[reserve_index];
        require(
            reserve.get_balance() == Decimal::zero(),
            "Reserve is not empty",
        );

        self.reserves.swap_remove(reserve_index);
    }

    pub fn deposit(
        &mut self,
        auth: StoreAuth<Self>,
        amount: Decimal,

        program_id: &Pubkey,
        collateral: &mut Collateral,

        user_account: Signer,
        source_token_account: TokenAccount<Writable>,
        token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
    ) {
        _ = auth;
        require(self.auction.is_none(), "Vault is liquidated");
        let reserve = self
            .reserves
            .iter_mut()
            .find(|r| r.get_mint() == collateral.get_mint())
            .ok_or("Reserve not found")
            .unwrap();
        reserve.deposit(
            amount,
            program_id,
            collateral,
            user_account,
            source_token_account,
            token_account,
            token_program_account,
        );
    }

    pub fn withdraw(
        &mut self,
        auth: StoreAuth<Self>,
        requested_amount: Decimal,

        debt_book: &mut Book,
        debt_config: &BookConfig,
        max_ltv: Decimal,

        program_id: &Pubkey,

        program_token_account: TokenAccount<Writable>,
        destination_token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,

        authority: Authority,

        mut collateral: Vec<&mut Collateral>,
        oracle_accounts: &[Readonly],
        reserve_index: usize,

        clock: &Clock,
    ) {
        _ = auth;
        require(self.auction.is_none(), "Vault is liquidated");
        require(reserve_index < self.reserves.len(), "Invalid reserve index");

        let collateral_value = {
            let mut sum = Decimal::zero();
            for ((&r, c), &o) in self
                .reserves
                .iter()
                .zip(collateral.iter())
                .zip(oracle_accounts)
            {
                sum += r.get_value(c, o, clock);
            }
            sum
        };
        let debt = self.debt.get_total(debt_book, debt_config, clock);
        let max_withdraw_value = collateral_value.saturating_sub(debt / max_ltv);

        let reserve_oracle = oracle_accounts[reserve_index];
        let reserve_collateral_price = collateral[reserve_index].get_price(reserve_oracle, clock);
        let max_withdraw_amount = max_withdraw_value / reserve_collateral_price;

        let reserve = &mut self.reserves[reserve_index];
        let amount = requested_amount
            .min(max_withdraw_amount)
            .min(reserve.get_balance());
        require(amount > Decimal::zero(), "Amount must be greater than zero");

        reserve.withdraw(
            amount,
            program_id,
            collateral[reserve_index],
            program_token_account,
            destination_token_account,
            token_program_account,
            authority,
        )
    }

    pub fn borrow(
        &mut self,
        auth: StoreAuth<Self>,
        requested_amount: Decimal,

        debt_book: &mut Book,
        debt_config: &BookConfig,
        dvd: &mut Token,
        max_ltv: Decimal,
        authority: Authority,

        collateral: &[&Collateral],
        oracle_accounts: &[Readonly],
        mint_account: MintAccount<Writable>,
        dvd_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
        clock: &Clock,
    ) {
        _ = auth;
        require(self.auction.is_none(), "Vault is liquidated");

        let collateral_value = {
            let mut sum = Decimal::zero();
            for ((&r, &c), &o) in self.reserves.iter().zip(collateral).zip(oracle_accounts) {
                sum += r.get_value(c, o, clock);
            }
            sum
        };

        let borrow_limit = collateral_value * max_ltv;
        let debt = self.debt.get_total(debt_book, debt_config, clock);
        let available_borrow = borrow_limit.saturating_sub(debt);

        let amount = available_borrow.min(requested_amount);

        self.debt.add(amount, debt_book, debt_config, clock);
        dvd.mint(
            amount,
            mint_account,
            dvd_account,
            authority,
            token_program_account,
        );
    }

    // Can be called during liquidation to repay debt and reduce collateral loss.
    pub fn repay(
        &mut self,
        auth: StoreAuth<Self>,
        requested_amount: Decimal,

        debt_book: &mut Book,
        debt_config: &BookConfig,
        dvd: &mut Token,

        user_account: Signer,
        mint_account: MintAccount<Writable>,
        dvd_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
        clock: &Clock,
    ) {
        _ = auth;
        let debt = self.debt.get_total(debt_book, debt_config, clock);
        let amount = requested_amount.min(debt);
        self.debt.subtract(amount, debt_book, debt_config, clock);
        dvd.burn(
            amount,
            mint_account,
            dvd_account,
            token_program_account,
            user_account,
        );
    }

    pub fn claim_rewards(
        &mut self,
        auth: StoreAuth<Self>,
        dove: &mut Token,
        dove_mint_account: MintAccount<Writable>,
        dove_token_account: TokenAccount<Writable>,
        authority: Authority,
        token_program_account: TokenProgramAccount,
        debt_book: &mut Book,
        debt_config: &BookConfig,
        clock: &Clock,
    ) {
        _ = auth;
        require(self.auction.is_none(), "Vault is liquidated");
        let amount = self.debt.claim_rewards(debt_book, debt_config, clock);
        if !amount.is_zero() {
            dove.mint(
                amount,
                dove_mint_account,
                dove_token_account,
                authority,
                token_program_account,
            );
        }
    }
}

// Unauthorized functions
impl Vault {
    pub fn liquidate(
        &mut self,

        max_ltv: Decimal,
        debt_book: &mut Book,
        debt_config: &BookConfig,
        vault_config: &VaultConfig,
        dvd: &mut Token,

        collateral: &[&Collateral],
        oracle_accounts: &[Readonly],

        dvd_mint_account: MintAccount<Writable>,
        dvd_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,

        authority: Authority,
        clock: &Clock,
    ) {
        require(self.auction.is_none(), "Vault is already liquidated");
        let collateral_value = {
            let mut sum = Decimal::zero();
            for ((&r, &c), &o) in self.reserves.iter().zip(collateral).zip(oracle_accounts) {
                sum += r.get_value(c, o, clock);
            }
            sum
        };
        let max_debt = collateral_value * max_ltv;
        let debt = self.debt.get_total(debt_book, debt_config, clock);
        if debt <= max_debt {
            revert("Vault is not unhealthy");
        }
        let mut auction_market_prices = [Decimal::zero(); MAX_RESERVES];
        for (i, (&c, &o)) in collateral.iter().zip(oracle_accounts).enumerate() {
            auction_market_prices[i] = c.get_price(o, clock);
        }
        self.auction = Some(Auction::new(auction_market_prices, Time::now(clock)));
        let liquidation_penalty = debt * vault_config.liquidation_penalty_rate;
        self.debt
            .add(liquidation_penalty, debt_book, debt_config, clock);
        let liquidation_reward =
            (debt * vault_config.liquidation_reward_rate).min(vault_config.liquidation_reward_cap);
        dvd.mint(
            liquidation_reward,
            dvd_mint_account,
            dvd_account,
            authority,
            token_program_account,
        );
    }

    pub fn unliquidate(&mut self) {
        require(self.auction.is_some(), "Vault is not liquidated");
        if !self.debt.is_zero() {
            revert("Vault has debt");
        }
        self.auction = None;
    }

    pub fn fail_auction(
        &mut self,
        debt_book: &mut Book,
        debt_config: &BookConfig,
        vault_config: &VaultConfig,
        auction_config: &AuctionConfig,
        dvd: &mut Token,
        dvd_mint_account: MintAccount<Writable>,
        dvd_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
        authority: Authority,
        clock: &Clock,
    ) {
        let auction = match &self.auction {
            Some(auction) => auction,
            None => revert("Vault is not liquidated"),
        };
        let is_over = auction.is_over(auction_config, Time::now(clock))
            || self.reserves.iter().all(|r| r.get_balance().is_zero());
        if !is_over {
            revert("Auction is not over");
        }
        self.auction = None;
        let debt = self.debt.take(debt_book, debt_config, clock);
        let auction_failure_reward = (debt * vault_config.auction_failure_reward_rate)
            .min(vault_config.auction_failure_reward_cap);
        dvd.mint(
            auction_failure_reward,
            dvd_mint_account,
            dvd_account,
            authority,
            token_program_account,
        );
    }

    pub fn buy_collateral(
        &mut self,
        requested_dvd_amount: Decimal,

        program_id: &Pubkey,

        collateral: &mut Collateral,
        debt_book: &mut Book,
        debt_config: &BookConfig,
        dvd: &mut Token,
        auction_config: &AuctionConfig,

        user_account: Signer,

        dvd_account: TokenAccount<Writable>,
        dvd_mint_account: MintAccount<Writable>,

        safe_account: TokenAccount<Writable>,
        collateral_destination_token_account: TokenAccount<Writable>,

        token_program_account: TokenProgramAccount,

        collateral_index: usize,
        authority: Authority,
        clock: &Clock,
    ) {
        let auction = match &self.auction {
            Some(auction) => auction,
            None => revert("Vault is not liquidated"),
        };
        let auction_price =
            auction.calculate_price(auction_config, Time::now(clock), collateral_index);

        let reserve = &mut self.reserves[collateral_index];
        let max_collateral_amount = reserve.get_balance();
        let max_dvd_amount = self.debt.get_total(debt_book, debt_config, clock);

        let requested_collateral_amount = requested_dvd_amount / auction_price;

        // This is necessary to prevent rounding errors.
        let (collateral_amount, dvd_amount) = if requested_dvd_amount >= max_dvd_amount
            && (max_dvd_amount / auction_price) <= max_collateral_amount
        {
            // 1st priority: repay all debt, so user can unliquidate!
            (max_dvd_amount / auction_price, max_dvd_amount)
        } else if requested_collateral_amount >= max_collateral_amount
            && (max_collateral_amount * auction_price) <= max_dvd_amount
        {
            // 2nd priority: repay all collateral, so we can fail later if necessary!
            (max_collateral_amount, max_collateral_amount * auction_price)
        } else {
            // 3rd priority: buy a portion of the collateral and repay a portion of the debt
            (requested_collateral_amount, requested_dvd_amount)
        };

        reserve.withdraw(
            collateral_amount,
            program_id,
            collateral,
            safe_account,
            collateral_destination_token_account,
            token_program_account,
            authority,
        );

        dvd.burn(
            dvd_amount,
            dvd_mint_account,
            dvd_account,
            token_program_account,
            user_account,
        );

        self.debt
            .subtract(dvd_amount, debt_book, debt_config, clock);
    }
}

// External functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
#[allow(non_snake_case)]
impl Vault {
    #[wasm_bindgen(js_name = deriveKey)]
    pub fn derive_key(programKey: &[u8], userKey: &[u8]) -> Result<Vec<u8>, String> {
        use crate::util::b2pk;
        Ok(Self::derive_address_raw(
            &b2pk(&programKey)?,
            &b2pk(&userKey)?,
        ))
    }

    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<Vault, String> {
        Self::try_cast_from(bytes)
            .map(|x| *x)
            .map_err(|e| format!("Invalid vault: {}", e))
    }

    #[wasm_bindgen(getter)]
    pub fn reserves(&self) -> Vec<Reserve> {
        self.reserves.to_vec()
    }

    #[wasm_bindgen(getter)]
    pub fn debt(&self) -> Page {
        self.debt
    }

    #[wasm_bindgen(getter, js_name = isDebtZero)]
    pub fn is_debt_zero(&self) -> bool {
        self.debt.is_zero()
    }

    #[wasm_bindgen(getter, js_name = isLiquidated)]
    pub fn is_liquidated(&self) -> bool {
        self.auction.is_some()
    }

    #[wasm_bindgen(js_name = calculateAuctionPrice)]
    pub fn calculate_auction_price(
        &self,
        index: usize,
        unixTimestamp: f64,
        config: &AuctionConfig,
    ) -> Option<f64> {
        self.auction.as_ref().map(|auction| {
            auction
                .calculate_price(
                    config,
                    Time::from_unix_timestamp(unixTimestamp as u64),
                    index,
                )
                .to_f64()
        })
    }

    #[wasm_bindgen(js_name = getAuctionSecsElapsed)]
    pub fn get_auction_secs_elapsed(&self, unixTimestamp: f64) -> Option<f64> {
        self.auction.as_ref().map(|auction| {
            auction.get_secs_elapsed(Time::from_unix_timestamp(unixTimestamp as u64)) as f64
        })
    }

    #[wasm_bindgen(js_name = isAuctionOver)]
    pub fn is_auction_over(&self, unixTimestamp: f64, config: &AuctionConfig) -> Option<bool> {
        self.auction.as_ref().map(|auction| {
            auction.is_over(config, Time::from_unix_timestamp(unixTimestamp as u64))
                || self.reserves.iter().all(|r| r.get_balance().is_zero())
        })
    }

    #[wasm_bindgen(js_name = getAuctionFailPrice)]
    pub fn get_auction_fail_price(&self, config: &AuctionConfig, index: usize) -> Option<f64> {
        self.auction
            .as_ref()
            .map(|auction| auction.get_fail_price(config, index).to_f64())
    }
}

unsafe impl Pod for Vault {
    const NAME: &'static str = "Vault";
}
