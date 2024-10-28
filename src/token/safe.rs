use {
    crate::{
        accounts::{
            MintAccount, Readonly, Signer, SystemProgramAccount, TokenAccount, TokenProgramAccount,
            Writable,
        },
        state::SovereignAuth,
        store::Authority,
        token::Mint,
        traits::{Account, Pod},
        util::require,
    },
    solana_program::{
        program::{invoke, invoke_signed},
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction,
    },
};

/// A safe place to store tokens.
/// The accounts where the protocol stores collateral.
#[derive(Clone, Copy)]
pub struct Safe {
    account: TokenAccount<Writable>,
    nonce: u8,
}

impl Safe {
    const SEED_PREFIX: &'static [u8] = b"safe";
    const SIZE: usize = spl_token::state::Account::LEN;
    pub fn create(
        program_id: &Pubkey,

        user_account: Signer,
        safe_account: Writable,
        mint_account: MintAccount<Readonly>,
        system_program_account: SystemProgramAccount,
        token_program_account: TokenProgramAccount,

        authority: Authority,

        rent: &Rent,

        sovereign_auth: SovereignAuth,
    ) -> Self {
        _ = system_program_account; // required for the instruction
        _ = token_program_account; // required for the instruction
        _ = sovereign_auth; // ensure that we have sovereign_auth authorization
        let user_info = user_account.get_info();
        let safe_account_info = safe_account.get_info();
        let mint_info = mint_account.get_info();
        let (key, nonce) = Pubkey::find_program_address(
            &[Self::SEED_PREFIX, mint_info.key.as_bytes()],
            program_id,
        );
        require(
            &key == safe_account_info.key,
            "Invalid safe account address",
        );
        invoke_signed(
            &system_instruction::create_account(
                user_info.key,
                safe_account_info.key,
                rent.minimum_balance(Self::SIZE),
                Self::SIZE as u64,
                &spl_token::ID,
            ),
            &[user_info.clone(), safe_account_info.clone()],
            &[&[Self::SEED_PREFIX, mint_info.key.as_bytes(), &[nonce]]],
        )
        .map_err(|_| "Failed to create safe account")
        .unwrap();
        invoke_signed(
            &spl_token::instruction::initialize_account3(
                &spl_token::ID,
                safe_account_info.key,
                mint_info.key,
                authority.get_account().get_info().key,
            )
            .map_err(|_| "Failed to create initialize token account instruction")
            .unwrap(),
            &[safe_account_info.clone(), mint_info.clone()],
            &[&[Self::SEED_PREFIX, mint_info.key.as_bytes(), &[nonce]]],
        )
        .map_err(|_| "Failed to initialize safe as token account")
        .unwrap();
        require(
            rent.is_exempt(safe_account_info.lamports(), Self::SIZE),
            "Safe account is not rent-exempt",
        );
        Self {
            account: TokenAccount::new(safe_account),
            nonce,
        }
    }
    pub fn get_nonce(&self) -> u8 {
        self.nonce
    }
    pub fn get(
        program_id: &Pubkey,
        safe_account: TokenAccount<Writable>,
        safe_account_nonce: u8,
        mint: &Mint,
    ) -> Self {
        let key = Pubkey::create_program_address(
            &[
                Self::SEED_PREFIX,
                mint.get_key().as_bytes(),
                &[safe_account_nonce],
            ],
            program_id,
        )
        .map_err(|_| "Failed to create safe account address")
        .unwrap();
        require(
            &key == safe_account.get_info().key,
            "Invalid safe account address",
        );
        Self {
            account: safe_account,
            nonce: safe_account_nonce,
        }
    }
    pub fn receive(
        &self,
        amount: u64,
        user_account: Signer,
        source_token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
    ) {
        let user_account_info = user_account.get_info();
        let source_token_account_info = source_token_account.get_info();
        let token_account_info = self.account.get_info();
        _ = token_program_account; // required for the instruction
        invoke(
            &spl_token::instruction::transfer(
                &spl_token::ID,
                source_token_account_info.key,
                token_account_info.key,
                user_account_info.key,
                &[],
                amount,
            )
            .map_err(|_| "couldn't create transfer instruction")
            .unwrap(),
            &[
                source_token_account_info.clone(),
                token_account_info.clone(),
                user_account_info.clone(),
            ],
        )
        .map_err(|_| "couldn't transfer tokens")
        .unwrap()
    }
    pub fn send(
        &self,
        amount: u64,
        destination_token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
        authority: Authority,
    ) {
        let authority_info = authority.get_account().get_info();
        let token_account_info = self.account.get_info();
        let destination_token_account_info = destination_token_account.get_info();
        _ = token_program_account; // required for the instruction
        invoke_signed(
            &spl_token::instruction::transfer(
                &spl_token::ID,
                token_account_info.key,
                destination_token_account_info.key,
                authority_info.key,
                &[],
                amount,
            )
            .map_err(|_| "couldn't create transfer instruction")
            .unwrap(),
            &[
                token_account_info.clone(),
                destination_token_account_info.clone(),
                authority_info.clone(),
            ],
            &[&authority.get_seeds()],
        )
        .map_err(|_| "couldn't transfer tokens")
        .unwrap()
    }
}

#[cfg(feature = "wasm")]
impl Safe {
    pub fn derive_address(program_id: &Pubkey, mint: &Pubkey) -> Pubkey {
        let (key, _) =
            Pubkey::find_program_address(&[Self::SEED_PREFIX, mint.as_bytes()], program_id);
        key
    }
}
