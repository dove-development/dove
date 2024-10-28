#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;
use {
    crate::{
        accounts::{Readonly, Signer, SystemProgramAccount, Writable},
        traits::{Account, Pod, Store},
    },
    solana_program::{pubkey::Pubkey, rent::Rent},
};

#[repr(C)]
#[derive(Clone, Copy)]
struct AuthorityStore {
    initialized: bool,
    nonce: u8,
}

impl Store for AuthorityStore {
    const SEED_PREFIX: &'static str = "authority";

    type Params = ();
    type DeriveData<'a> = ();
    type CreateData<'a> = ();
    type LoadData = ();
    type LoadAuthData = ();

    fn get_seeds_on_derive<'a>(_: Self::DeriveData<'a>) -> [&'a [u8]; 2] {
        [&[], &[]]
    }
    fn get_seeds_on_create<'a>(_: Self::CreateData<'a>) -> [&'a [u8]; 2] {
        [&[], &[]]
    }
    fn get_seeds_on_load(&self, _: Self::LoadData) -> [&'static [u8]; 2] {
        [&[], &[]]
    }
    fn get_seeds_on_load_auth(&self, _: Self::LoadAuthData) -> [&'static [u8]; 2] {
        [&[], &[]]
    }
    fn initialize<'a>(&mut self, nonce: u8, _: Self::Params) {
        self.initialized = true;
        self.nonce = nonce;
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn get_nonce(&self) -> u8 {
        self.nonce
    }
}

unsafe impl Pod for AuthorityStore {
    const NAME: &'static str = "Authority";
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Authority {
    account: Readonly,
    nonce_array: [u8; 1],
}

impl Authority {
    pub fn create(
        program_id: &Pubkey,

        user_account: Signer,
        authority_account: Writable,
        system_program_account: SystemProgramAccount,

        rent: &Rent,
    ) {
        AuthorityStore::create(
            program_id,
            user_account,
            authority_account,
            system_program_account,
            (),
            rent,
            (),
        );
    }
    pub fn from_account(program_id: &Pubkey, authority_account: Readonly) -> Self {
        let store_data = authority_account.get_info().data.borrow();
        let authority = AuthorityStore::load(program_id, authority_account, &store_data, ());
        Self {
            account: authority_account,
            nonce_array: [authority.get_nonce()],
        }
    }
    pub fn get_account(&self) -> Readonly {
        self.account
    }
    pub fn get_seeds(&self) -> [&[u8]; 2] {
        [AuthorityStore::SEED_PREFIX.as_bytes(), &self.nonce_array]
    }
}

#[cfg(feature = "wasm")]
impl Authority {
    pub fn derive_address(program_id: &Pubkey) -> Pubkey {
        AuthorityStore::derive_address(&program_id, ())
    }
}

// External functions
#[cfg(feature = "wasm")]
#[allow(non_snake_case)]
#[wasm_bindgen]
impl Authority {
    #[wasm_bindgen(js_name = deriveKey)]
    pub fn derive_key(program_key: &[u8]) -> Result<Vec<u8>, String> {
        use crate::util::b2pk;
        Ok(AuthorityStore::derive_address_raw(&b2pk(program_key)?, ()))
    }
}
