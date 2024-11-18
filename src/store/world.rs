use {
    crate::{
        accounts::{MintAccount, Readonly, Signer},
        finance::{Book, Schedule},
        state::{Config, DvdPrice, FlashMint, Offering, Sovereign, StableDvd, Vesting},
        store::Authority,
        token::Token,
        traits::{Pod, Store},
        util::{require, Expect},
    },
    solana_program::{clock::Clock, pubkey::Pubkey},
};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// The struct containing all global state.
///
/// **Safety**: Never overwrite the fields of this struct directly (/world\.(.+) = /).
/// Doing so is unsafe and violates critical invariants.
/// Always use the associated functions on the state objects to modify state.
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct World {
    initialized: bool,
    nonce: u8,

    pub dove: Token,
    pub dvd: Token,
    pub debt: Book,
    pub savings: Book,

    pub stable_dvd: StableDvd,
    pub dvd_price: DvdPrice,

    pub offering: Offering,
    pub flash_mint: FlashMint,
    pub sovereign: Sovereign,
    pub vesting: Vesting,

    pub config: Config,
}

pub struct WorldParams {
    pub dove_mint_account: MintAccount<Readonly>,
    pub dvd_mint_account: MintAccount<Readonly>,
    pub sovereign_account: Signer,
    pub vesting_recipient: Pubkey,
    pub vesting_schedule: Schedule,
    pub authority: Authority,
    pub clock: Clock,
}

impl Store for World {
    const SEED_PREFIX: &'static str = "world";
    type Params = WorldParams;
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
    fn get_seeds_on_load_auth(&self, _: ()) -> [&'static [u8]; 2] {
        [&[], &[]]
    }
    fn initialize(&mut self, nonce: u8, params: WorldParams) {
        require(!self.initialized, "World is already initialized");
        self.initialized = true;
        self.nonce = nonce;
        self.dove = Token::from_account(
            params.dove_mint_account,
            params.authority,
            Expect::None,
            Expect::None,
        );
        self.dvd = Token::from_account(
            params.dvd_mint_account,
            params.authority,
            Expect::None,
            Expect::None,
        );
        self.debt = Book::new(&params.clock);
        self.savings = Book::new(&params.clock);
        self.stable_dvd = StableDvd::new();
        self.dvd_price = DvdPrice::new(&params.clock);
        self.config = Config::zero();
        self.sovereign = Sovereign::new(params.sovereign_account);
        self.offering = Offering::new();
        self.flash_mint = FlashMint::new();
        self.vesting = Vesting::new(
            &params.clock,
            params.vesting_recipient,
            params.vesting_schedule,
        );
    }
    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn get_nonce(&self) -> u8 {
        self.nonce
    }
}

unsafe impl Pod for World {
    const NAME: &'static str = "World";
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl World {
    #[wasm_bindgen(js_name = deriveKey)]
    pub fn derive_key(program_key: Vec<u8>) -> Result<Vec<u8>, String> {
        use crate::util::b2pk;
        Ok(Self::derive_address_raw(&b2pk(&program_key)?, ()))
    }

    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<World, String> {
        Self::try_cast_from(bytes)
            .map(|x| *x)
            .map_err(|e| format!("Invalid world: {}", e))
    }

    #[wasm_bindgen(js_name = zero)]
    pub fn zero_wasm() -> World {
        Self::zero()
    }

    #[wasm_bindgen(getter, js_name = dvdPrice)]
    pub fn dvd_price_wasm(&self) -> DvdPrice {
        self.dvd_price
    }

    #[wasm_bindgen(getter, js_name = stableDvd)]
    pub fn stable_dvd_wasm(&self) -> StableDvd {
        self.stable_dvd
    }

    #[wasm_bindgen(getter, js_name = flashMint)]
    pub fn flash_mint_wasm(&self) -> FlashMint {
        self.flash_mint
    }
}
