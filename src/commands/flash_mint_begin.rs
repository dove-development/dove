#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    solana_program::sysvar::instructions,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{
            InstructionSysvarAccount, MintAccount, Readonly, TokenAccount, TokenProgramAccount,
            Writable,
        },
        commands::FlashMintEnd,
        finance::Decimal,
        keys::{DvdMintKey, UserKey},
        store::{Authority, World},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};

/// Executes a flash mint operation
///
/// Accounts expected:
///
/// 0. `[writable]` World account (PDA)
/// 1. `[writable]` Debt token mint account
/// 2. `[writable]` User's DVD account
/// 3. `[]` Authority account (PDA)
/// 4. `[]` SPL Token program
/// 5. `[]` Instruction sysvar account
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct FlashMintBegin {
    borrow_amount: Decimal,
}

#[cfg(feature = "wasm")]
#[allow(non_snake_case)]
#[wasm_bindgen]
impl FlashMintBegin {
    #[wasm_bindgen(js_name = "getData")]
    #[allow(non_snake_case)]
    pub fn get_data_wasm(borrowAmount: f64) -> Vec<u8> {
        Self {
            borrow_amount: Decimal::from(borrowAmount),
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        dvdMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let userKey = UserKey::new(b2pk(userKey)?);
        let dvdMintKey = DvdMintKey::new(b2pk(dvdMintKey)?);
        let accounts = Self::get_accounts(programKey, (userKey, dvdMintKey))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for FlashMintBegin {}

impl Command for FlashMintBegin {
    const ID: u32 = 0x3bf81d03;
    type Keys = (UserKey, DvdMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, dvd_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *dvd_mint_key,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: user_key.derive_associated_token_address(&dvd_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_authority(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: spl_token::id(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: instructions::ID,
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let world_account = Writable::new(&accounts[0]);
        let dvd_mint_account = MintAccount::new(Writable::new(&accounts[1]));
        let dvd_account = TokenAccount::new(Writable::new(&accounts[2]));
        let authority_account = Readonly::new(&accounts[3]);
        let token_program_account = TokenProgramAccount::new(&accounts[4]);
        let instruction_sysvar_account = InstructionSysvarAccount::new(&accounts[5]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let authority = Authority::from_account(program_id, authority_account);

        let flash_mint_end_instruction_data = FlashMintEnd::ID.to_le_bytes();

        world.flash_mint.begin(
            self.borrow_amount,
            &flash_mint_end_instruction_data,
            program_id,
            authority,
            dvd_mint_account,
            dvd_account,
            token_program_account,
            instruction_sysvar_account,
            world.config.get_flash_mint_config(),
            &mut world.dvd,
        );
    }
}
