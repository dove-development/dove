#[cfg(feature = "wasm")]
use {
    crate::util::b2pk,
    wasm_bindgen::prelude::wasm_bindgen
};
use {
    crate::{
        accounts::Signer,
        finance::Decimal,
        traits::{Account, Pod, Store, StoreAuth},
        util::Time,
    },
    solana_program::pubkey::Pubkey
};

/// A user-controlled oracle.
///
/// It's useful for:
/// 1. Debugging: Developers can set custom prices for testing.
/// 2. Initial implementation: For newly launched tokens or assets without established external price feeds.
///    For example, when this program is first launched, DOVE will not have an external price feed.
/// 3. Flexibility: Other account types can be deserialized as UserFeed,
///    allowing easy extension of oracle functionality
///    without modifying the core implementation.
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct UserFeed {
    initialized: bool,
    nonce: u8,
    index: [u8; 1],
    price: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl UserFeed {
    pub fn derive_key(program_key: &[u8], user_key: &[u8], index: u8) -> Result<Vec<u8>, String> {
        Ok(Self::derive_address_raw(&b2pk(&program_key)?, (&b2pk(&user_key)?, &[index])))
    }
}

impl Store for UserFeed {
    const SEED_PREFIX: &'static str = "user_feed";

    type Params = u8;
    type DeriveData<'a> = (&'a Pubkey, &'a [u8; 1]);
    type CreateData<'a> = (Signer, &'a [u8; 1]);
    type LoadData = ();
    type LoadAuthData = Signer;

    fn get_seeds_on_derive<'a>((program_key, index): Self::DeriveData<'a>) -> [&'a [u8]; 2] {
        [program_key.as_bytes(), index]
    }
    fn get_seeds_on_create<'a>((user_account, index): Self::CreateData<'a>) -> [&'a [u8]; 2] {
        [user_account.get_info().key.as_bytes(), index]
    }
    fn get_seeds_on_load(&self, _: ()) -> [&'static [u8]; 2] {
        unimplemented!("UserFeed does not have an unprivileged mode")
    }
    fn get_seeds_on_load_auth(&self, user_account: Self::LoadAuthData) -> [&[u8]; 2] {
        [user_account.get_info().key.as_bytes(), &self.index]
    }

    fn initialize(&mut self, nonce: u8, index: u8) {
        self.initialized = true;
        self.nonce = nonce;
        self.index = [index];
        self.price = Decimal::zero();
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn get_nonce(&self) -> u8 {
        self.nonce
    }
}

impl UserFeed {
    pub fn set_price(&mut self, auth: StoreAuth<Self>, price: Decimal) {
        _ = auth;
        self.price = price;
    }
}

unsafe impl Pod for UserFeed {
    const NAME: &'static str = "UserFeed";
}

impl UserFeed {
    pub fn query(data: &[u8], time: Time) -> Result<(Decimal, Time), &'static str> {
        // Provided oracle account key assumed valid
        let user_feed = Self::load_unchecked(&data)?;
        Ok((user_feed.price, time))
    }
}