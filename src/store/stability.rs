#[cfg(feature = "wasm")]
use {crate::util::b2pk, wasm_bindgen::prelude::wasm_bindgen};
use {
    crate::{
        accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::Decimal,
        state::{SovereignAuth, StableDvd},
        store::Authority,
        token::{Mint, Safe, Token},
        traits::{Account, Pod, Store, StoreAuth},
        util::{revert, Expect},
    },
    solana_program::pubkey::Pubkey,
};

/// A liquidity pool allowing 1:1 swapping between on-demand minted DVD and a
/// blue-chip stablecoin. This helps to stabilize the market price of DVD.
///
/// To protect against depegs, a `mint_limit` is set by governance: the maximum
/// amount, in USD, that the protocol is willing to lose in the event of a depeg.
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Stability {
    initialized: bool,
    nonce: u8,
    safe_nonce: u8,
    mint_decimals: u8,
    stable_mint: Mint,
    mint_limit: Decimal,
    minted: Decimal,
}

pub struct StabilityParams {
    pub stable_mint_account: MintAccount<Readonly>,
    pub sovereign_auth: SovereignAuth,
    pub safe_nonce: u8,
}

impl Store for Stability {
    const SEED_PREFIX: &'static str = "stability";
    type Params = StabilityParams;
    type DeriveData<'a> = &'a Pubkey;
    type CreateData<'a> = MintAccount<Readonly>;
    type LoadData = ();
    type LoadAuthData = SovereignAuth;

    fn get_seeds_on_derive<'a>(stable_mint: Self::DeriveData<'a>) -> [&'a [u8]; 2] {
        [stable_mint.as_bytes(), &[]]
    }
    fn get_seeds_on_create<'a>(stable_mint_account: MintAccount<Readonly>) -> [&'a [u8]; 2] {
        [stable_mint_account.get_info().key.as_bytes(), &[]]
    }
    fn get_seeds_on_load<'a>(&'a self, _: ()) -> [&'a [u8]; 2] {
        [self.stable_mint.get_key().as_bytes(), &[]]
    }
    fn get_seeds_on_load_auth<'a>(&'a self, sovereign_auth: SovereignAuth) -> [&'a [u8]; 2] {
        _ = sovereign_auth;
        [self.stable_mint.get_key().as_bytes(), &[]]
    }

    fn initialize(&mut self, nonce: u8, params: Self::Params) {
        _ = params.sovereign_auth;
        self.initialized = true;
        self.nonce = nonce;
        self.safe_nonce = params.safe_nonce;
        self.stable_mint = Mint::from_account(
            params.stable_mint_account,
            Expect::Any,
            Expect::Any,
            Expect::Any,
            &mut self.mint_decimals,
            &mut 0
        );
        self.mint_limit = Decimal::zero();
        self.minted = Decimal::zero();
    }
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    fn get_nonce(&self) -> u8 {
        self.nonce
    }
}

// Authorized functions
impl Stability {
    pub fn update_mint_limit(&mut self, _: StoreAuth<Self>, mint_limit: Decimal) {
        self.mint_limit = mint_limit;
    }
}

impl Stability {
    pub fn buy_dvd(
        &mut self,
        amount: Decimal,
        dvd: &mut Token,
        stable_dvd: &mut StableDvd,
        program_id: &Pubkey,
        authority: Authority,
        user_account: Signer,
        safe_account: TokenAccount<Writable>,
        stable_token_account: TokenAccount<Writable>,
        dvd_token_account: TokenAccount<Writable>,
        dvd_mint_account: MintAccount<Writable>,
        token_program_account: TokenProgramAccount,
    ) {
        if amount.is_zero() {
            return;
        }
        let new_minted_amount = self.minted + amount;
        if new_minted_amount > self.mint_limit {
            revert("mint limit exceeded");
        }

        let safe = Safe::get(program_id, safe_account, self.safe_nonce, &self.stable_mint);
        (safe).receive(
            amount.to_token_amount(self.mint_decimals),
            user_account,
            stable_token_account,
            token_program_account,
        );
        dvd.mint(
            amount,
            dvd_mint_account,
            dvd_token_account,
            authority,
            token_program_account,
        );
        stable_dvd.increase(amount);

        self.minted = new_minted_amount;
    }

    pub fn sell_dvd(
        &mut self,
        amount: Decimal,
        dvd: &mut Token,
        stable_dvd: &mut StableDvd,
        program_id: &Pubkey,
        authority: Authority,
        user_account: Signer,
        safe_account: TokenAccount<Writable>,
        stable_token_account: TokenAccount<Writable>,
        dvd_token_account: TokenAccount<Writable>,
        dvd_mint_account: MintAccount<Writable>,
        token_program_account: TokenProgramAccount,
    ) {
        if amount.is_zero() {
            return;
        }
        if amount > self.minted {
            revert("not enough stablecoin available to swap to");
        }
        let new_minted_amount = self.minted - amount;

        let safe = Safe::get(program_id, safe_account, self.safe_nonce, &self.stable_mint);

        (safe).send(
            amount.to_token_amount(self.mint_decimals),
            stable_token_account,
            token_program_account,
            authority,
        );
        dvd.burn(
            amount,
            dvd_mint_account,
            dvd_token_account,
            token_program_account,
            user_account,
        );
        stable_dvd.decrease(amount);
        
        self.minted = new_minted_amount;
    }
}

// External functions
#[cfg(feature = "wasm")]
#[allow(non_snake_case)]
#[wasm_bindgen]
impl Stability {
    #[wasm_bindgen(js_name = deriveKey)]
    #[allow(non_snake_case)]
    pub fn derive_key(programKey: &[u8], stableMintKey: &[u8]) -> Result<Vec<u8>, String> {
        Ok(Self::derive_address_raw(
            &b2pk(&programKey)?,
            &b2pk(&stableMintKey)?,
        ))
    }

    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<Stability, String> {
        Self::try_cast_from(bytes)
            .map(|x| *x)
            .map_err(|e| format!("Invalid stability: {}", e))
    }

    #[wasm_bindgen(getter, js_name = minted)]
    pub fn minted(&self) -> f64 {
        self.minted.to_f64()
    }

    #[wasm_bindgen(getter, js_name = mintLimit)]
    pub fn mint_limit(&self) -> f64 {
        self.mint_limit.to_f64()
    }

    #[wasm_bindgen(getter, js_name = mintKey)]
    pub fn mint_key(&self) -> Vec<u8> {
        self.stable_mint.get_key().to_bytes().to_vec()
    }
}

unsafe impl Pod for Stability {
    const NAME: &'static str = "Stability";
}
