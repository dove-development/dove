use crate::{
    finance::{Book, BookConfig},
    util::revert,
};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use {
    crate::{
        accounts::{MintAccount, Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::{Decimal, Page},
        store::Authority,
        token::Token,
        traits::{Account, Pod, Store, StoreAuth},
    },
    solana_program::{clock::Clock, pubkey::Pubkey},
};

#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Savings {
    initialized: bool,
    nonce: u8,
    page: Page,
}

impl Store for Savings {
    const SEED_PREFIX: &'static str = "savings";

    type Params = ();
    type DeriveData<'a> = &'a Pubkey;
    type CreateData<'a> = Signer;
    type LoadData = ();
    type LoadAuthData = Signer;

    fn get_seeds_on_derive<'a>(derive_data: Self::DeriveData<'a>) -> [&'a [u8]; 2] {
        [derive_data.as_bytes(), &[]]
    }
    fn get_seeds_on_create<'a>(user_account: Signer) -> [&'a [u8]; 2] {
        [user_account.get_info().key.as_bytes(), &[]]
    }
    fn get_seeds_on_load(&self, _: ()) -> [&'static [u8]; 2] {
        unimplemented!("Savings does not have an unprivileged mode")
    }
    fn get_seeds_on_load_auth(&self, user_account: Signer) -> [&'static [u8]; 2] {
        [user_account.get_info().key.as_bytes(), &[]]
    }

    fn initialize(&mut self, nonce: u8, _: Self::Params) {
        self.initialized = true;
        self.nonce = nonce;
        self.page = Page::new();
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn get_nonce(&self) -> u8 {
        self.nonce
    }
}

// Authorized functions
impl Savings {
    pub fn deposit(
        &mut self,
        auth: StoreAuth<Self>,
        amount: Decimal,
        dvd: &mut Token,
        savings_book: &mut Book,
        savings_config: &BookConfig,

        user_account: Signer,
        dvd_mint_account: MintAccount<Writable>,
        dvd_token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
        clock: &Clock,
    ) {
        _ = auth;
        dvd.burn(
            amount,
            dvd_mint_account,
            dvd_token_account,
            token_program_account,
            user_account,
        );
        self.page.add(amount, savings_book, savings_config, clock);
    }

    pub fn withdraw(
        &mut self,
        auth: StoreAuth<Self>,
        requested_amount: Decimal,
        savings_book: &mut Book,
        savings_config: &BookConfig,
        dvd: &mut Token,

        dvd_mint_account: MintAccount<Writable>,
        dvd_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,

        authority: Authority,
        clock: &Clock,
    ) {
        _ = auth;
        let amount = requested_amount.min(self.page.get_total(savings_book, savings_config, clock));
        if amount.is_zero() {
            revert("Insufficient savings");
        }
        self.page
            .subtract(amount, savings_book, savings_config, clock);
        dvd.mint(
            amount,
            dvd_mint_account,
            dvd_account,
            authority,
            token_program_account,
        );
    }

    pub fn claim_rewards(
        &mut self,
        auth: StoreAuth<Self>,
        savings_book: &mut Book,
        savings_config: &BookConfig,
        dove: &mut Token,

        dove_mint_account: MintAccount<Writable>,
        dove_token_account: TokenAccount<Writable>,
        authority: Authority,
        token_program_account: TokenProgramAccount,

        clock: &Clock,
    ) {
        _ = auth;
        let amount = self.page.claim_rewards(savings_book, savings_config, clock);
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

// External functions
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Savings {
    #[wasm_bindgen(js_name = deriveKey)]
    #[allow(non_snake_case)]
    pub fn derive_key(programKey: &[u8], userKey: &[u8]) -> Result<Vec<u8>, String> {
        let program_key =
            <[u8; 32]>::try_from(programKey).map_err(|e| format!("Invalid program key: {}", e))?;
        let program_key = Pubkey::new_from_array(program_key);
        let user_key =
            <[u8; 32]>::try_from(userKey).map_err(|e| format!("Invalid user key: {}", e))?;
        let user_key = Pubkey::new_from_array(user_key);
        Ok(Self::derive_address_raw(&program_key, &user_key))
    }

    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<Savings, String> {
        Self::try_cast_from(bytes)
            .map(|x| *x)
            .map_err(|e| format!("Invalid savings: {}", e))
    }

    #[wasm_bindgen(getter)]
    pub fn page(&self) -> Page {
        self.page
    }
}

unsafe impl Pod for Savings {
    const NAME: &'static str = "Savings";
}
