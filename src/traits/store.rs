use {
    super::Account,
    crate::{
        accounts::{Readonly, Signer, SystemProgramAccount, Writable},
        traits::Pod,
        util::require,
    },
    solana_program::{program::invoke_signed, pubkey::Pubkey, rent::Rent, system_instruction},
    std::marker::PhantomData,
};

pub struct StoreAuth<T: Store> {
    _phantom: PhantomData<T>,
}

impl<T: Store> StoreAuth<T> {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

pub trait Store: Pod {
    const SEED_PREFIX: &'static str;
    type Params;

    type DeriveData<'a>;
    type CreateData<'a>;
    type LoadData;
    type LoadAuthData;

    #[allow(unused)]
    fn get_seeds_on_derive<'a>(derive_data: Self::DeriveData<'a>) -> [&'a [u8]; 2];
    fn get_seeds_on_create<'a>(create_data: Self::CreateData<'a>) -> [&'a [u8]; 2];
    fn get_seeds_on_load<'a>(&'a self, load_data: Self::LoadData) -> [&'a [u8]; 2];
    fn get_seeds_on_load_auth<'a>(&'a self, load_auth_data: Self::LoadAuthData) -> [&'a [u8]; 2];
    fn initialize(&mut self, nonce: u8, params: Self::Params);
    fn is_initialized(&self) -> bool;
    fn get_nonce(&self) -> u8;

    #[cfg(feature = "wasm")]
    fn derive_address<'a>(
        program_id: &Pubkey,
        derive_data: Self::DeriveData<'a>,
    ) -> Pubkey {
        let seeds = Self::get_seeds_on_derive(derive_data);
        let (key, _) = Pubkey::find_program_address(
            &[Self::SEED_PREFIX.as_bytes(), &seeds[0], seeds[1]],
            program_id,
        );
        key
    }

    #[cfg(feature = "wasm")]
    fn derive_address_raw<'a>(
        program_id: &Pubkey,
        derive_data: Self::DeriveData<'a>,
    ) -> Vec<u8> {
        Self::derive_address(program_id, derive_data).to_bytes().to_vec()
    }

    fn create<'a>(
        program_id: &Pubkey,
        funder_account: Signer,
        store_account: Writable,
        system_program_account: SystemProgramAccount,
        create_data: Self::CreateData<'a>,
        rent: &Rent,
        params: Self::Params,
    ) {
        let user_info = funder_account.get_info();
        let store_info = store_account.get_info();
        let seeds = Self::get_seeds_on_create(create_data);
        let (key, nonce) = Pubkey::find_program_address(
            &[Self::SEED_PREFIX.as_bytes(), &seeds[0], &seeds[1]],
            program_id,
        );

        require(
            &key == store_info.key,
            "Invalid store address",
        );

        _ = system_program_account;

        invoke_signed(
            &system_instruction::create_account(
                user_info.key,
                store_info.key,
                rent.minimum_balance(Self::SIZE),
                Self::SIZE as u64,
                program_id,
            ),
            &[user_info.clone(), store_info.clone()],
            &[&[Self::SEED_PREFIX.as_bytes(), &seeds[0], &seeds[1], &[nonce]]],
        )
        .map_err(|_| "Failed to create store account")
        .unwrap();

        require(
            rent.is_exempt(store_info.lamports(), Self::SIZE),
            "Account must be rent exempt"
        );

        let mut store_data = store_info.data.borrow_mut();
        let store = Self::cast_from_mut(&mut store_data[..]);
        store.initialize(nonce, params);

        require(store.is_initialized(), "Store should be initialized");
        require(
            store.get_nonce() == nonce,
            "Store did not store nonce correctly",
        );
    }
    fn load<'data>(
        program_id: &Pubkey,
        store_account: Readonly,
        store_data: &'data [u8],
        load_data: Self::LoadData,
    ) -> &'data Self {
        let store = Self::cast_from(store_data);
        require(store.is_initialized(), "Store is not initialized");

        let seeds = store.get_seeds_on_load(load_data);
        let key = Pubkey::create_program_address(
            &[Self::SEED_PREFIX.as_bytes(), &seeds[0], &seeds[1], &[store.get_nonce()]],
            program_id,
        )
        .map_err(|_| "Failed to create store address").unwrap();
        require(
            &key == store_account.get_info().key,
            "Invalid store address",
        );

        store
    }
    fn load_mut<'data>(
        program_id: &Pubkey,
        store_account: Writable,
        store_data: &'data mut [u8],
        load_data: Self::LoadData,
    ) -> &'data mut Self {
        let store = Self::cast_from_mut(store_data);
        require(store.is_initialized(), "Store is not initialized");

        let seeds = store.get_seeds_on_load(load_data);
        let key = Pubkey::create_program_address(
            &[Self::SEED_PREFIX.as_bytes(), &seeds[0], &seeds[1], &[store.get_nonce()]],
            program_id,
        )
        .map_err(|_| "Failed to create store address").unwrap();
        require(
            &key == store_account.get_info().key,
            "Invalid store address",
        );

        store
    }

    fn load_auth<'data>(
        program_id: &Pubkey,
        store_account: Writable,
        store_data: &'data mut [u8],
        load_auth_data: Self::LoadAuthData,
    ) -> (&'data mut Self, StoreAuth<Self>) {
        let store = Self::cast_from_mut(store_data);
        require(store.is_initialized(), "Store is not initialized");

        let seeds = store.get_seeds_on_load_auth(load_auth_data);
        let key = Pubkey::create_program_address(
            &[Self::SEED_PREFIX.as_bytes(), &seeds[0], &seeds[1], &[store.get_nonce()]],
            program_id,
        )
        .map_err(|_| "Failed to create store address").unwrap();
        require(
            &key == store_account.get_info().key,
            "Invalid store address",
        );

        (store, StoreAuth::new())
    }

    fn load_unchecked(store_data: &[u8]) -> Result<&Self, &'static str> {
        let store = Self::try_cast_from(store_data)?;
        if !store.is_initialized() {
            return Err("Store is not initialized");
        }
        Ok(store)
    }
}
