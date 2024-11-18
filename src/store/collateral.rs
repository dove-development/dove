#[cfg(feature = "wasm")]
use {crate::util::b2pk, wasm_bindgen::prelude::wasm_bindgen};
use {
    crate::{
        accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::{Decimal, InterestRate},
        oracle::Oracle,
        state::{DvdPrice, SovereignAuth},
        store::Authority,
        token::{Mint, Safe},
        traits::{Account, Pod, Store, StoreAuth},
        util::{revert, Expect},
    },
    solana_program::{clock::Clock, pubkey::Pubkey},
};

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Collateral {
    initialized: bool,
    nonce: u8,
    safe_nonce: u8,
    mint_decimals: u8,
    mint: Mint,
    deposited: Decimal,
    max_deposit: Decimal,
    oracle: Oracle,
}

pub struct CollateralParams {
    pub sovereign_auth: SovereignAuth,
    pub safe_nonce: u8,
    pub mint_account: MintAccount<Readonly>,
}

impl Store for Collateral {
    const SEED_PREFIX: &'static str = "collateral";
    type Params = CollateralParams;
    type DeriveData<'a> = &'a Pubkey;
    type CreateData<'a> = MintAccount<Readonly>;
    type LoadData = ();
    type LoadAuthData = SovereignAuth;

    fn get_seeds_on_derive<'a>(derive_data: Self::DeriveData<'a>) -> [&'a [u8]; 2] {
        [derive_data.as_bytes(), &[]]
    }
    fn get_seeds_on_create<'a>(mint_account: MintAccount<Readonly>) -> [&'a [u8]; 2] {
        [mint_account.get_info().key.as_bytes(), &[]]
    }
    fn get_seeds_on_load<'a>(&'a self, _: ()) -> [&'a [u8]; 2] {
        [self.mint.get_key().as_bytes(), &[]]
    }
    fn get_seeds_on_load_auth<'a>(&'a self, sovereign_auth: SovereignAuth) -> [&'a [u8]; 2] {
        _ = sovereign_auth;
        [self.mint.get_key().as_bytes(), &[]]
    }

    fn initialize(&mut self, nonce: u8, params: Self::Params) {
        _ = params.sovereign_auth;
        self.initialized = true;
        self.nonce = nonce;
        self.safe_nonce = params.safe_nonce;
        self.mint = Mint::from_account(
            params.mint_account,
            Expect::Any,
            Expect::Any,
            Expect::Any,
            &mut self.mint_decimals,
            &mut 0,
        );
        self.deposited = Decimal::zero();
        self.max_deposit = Decimal::zero();
        self.oracle = Oracle::zero();
    }
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    fn get_nonce(&self) -> u8 {
        self.nonce
    }
}

// Authorized functions
impl Collateral {
    pub fn update_max_deposit(&mut self, auth: StoreAuth<Self>, new_max_deposit: Decimal) {
        _ = auth;
        self.max_deposit = new_max_deposit;
    }
    pub fn set_oracle(&mut self, auth: StoreAuth<Self>, oracle: Oracle) {
        _ = auth;
        self.oracle = oracle;
    }
}

// Unauthorized functions
impl Collateral {
    pub const fn get_mint(&self) -> &Mint {
        &self.mint
    }
}

// For internal use only
impl Collateral {
    pub fn receive(
        &mut self,
        amount: Decimal,

        program_id: &Pubkey,

        user_account: Signer,
        source_token_account: TokenAccount<Writable>,
        program_token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
    ) {
        let token_account = Safe::get(
            program_id,
            program_token_account,
            self.safe_nonce,
            &self.mint,
        );
        let new_deposited = self.deposited + amount;
        if new_deposited > self.max_deposit {
            revert("Deposit limit for collateral type reached");
        }
        self.deposited = new_deposited;
        token_account.receive(
            amount.to_token_amount(self.mint_decimals),
            user_account,
            source_token_account,
            token_program_account,
        );
    }
    pub fn send(
        &mut self,
        amount: Decimal,

        program_id: &Pubkey,

        safe_account: TokenAccount<Writable>,
        destination_token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,

        authority: Authority,
    ) {
        let token_account = Safe::get(program_id, safe_account, self.safe_nonce, &self.mint);
        token_account.send(
            amount.to_token_amount(self.mint_decimals),
            destination_token_account,
            token_program_account,
            authority,
        );
        self.deposited -= amount;
    }
    pub fn get_price(
        &self,
        oracle_account: Readonly,
        dvd_price: &mut DvdPrice,
        dvd_interest_rate: &InterestRate,
        clock: &Clock,
    ) -> Decimal {
        self.oracle
            .query_dvd(oracle_account, dvd_price, dvd_interest_rate, clock)
    }
}

// External functions
#[cfg(feature = "wasm")]
#[allow(non_snake_case)]
#[wasm_bindgen]
impl Collateral {
    #[wasm_bindgen(js_name = deriveKey)]
    #[allow(non_snake_case)]
    pub fn derive_key(programKey: &[u8], collateralMintKey: &[u8]) -> Result<Vec<u8>, String> {
        Ok(Self::derive_address_raw(
            &b2pk(&programKey)?,
            &b2pk(&collateralMintKey)?,
        ))
    }

    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<Collateral, String> {
        Self::try_cast_from(bytes)
            .map(|x| *x)
            .map_err(|e| format!("Invalid collateral: {}", e))
    }

    #[wasm_bindgen(getter)]
    pub fn oracle(&self) -> Oracle {
        self.oracle
    }

    #[wasm_bindgen(getter)]
    pub fn decimals(&self) -> u8 {
        self.mint_decimals
    }

    #[wasm_bindgen(getter)]
    pub fn deposited(&self) -> f64 {
        self.deposited.to_f64()
    }

    #[wasm_bindgen(getter, js_name = maxDeposit)]
    pub fn max_deposit(&self) -> f64 {
        self.max_deposit.to_f64()
    }
}

unsafe impl Pod for Collateral {
    const NAME: &'static str = "Collateral";
}
