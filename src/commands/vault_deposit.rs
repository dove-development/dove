use {
    crate::{
        accounts::{Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::Decimal,
        keys::{CollateralMintKey, UserKey},
        store::{Collateral, Vault},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};
#[cfg(feature = "wasm")]
use {
    crate::{
        keys::ProgramKey,
        util::{b2pk, AccountWasm},
    },
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};

/// Deposits tokens into a vault
///
/// Accounts expected:
///
/// 0. `[signer]` User account
/// 1. `[writable]` Vault account (PDA)
/// 2. `[writable]` Collateral account (PDA)
/// 3. `[writable]` User's token account (source of tokens)
/// 4. `[writable]` Collateral's token account (destination for tokens)
/// 5. `[]` SPL Token program
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VaultDeposit {
    amount: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VaultDeposit {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm(amount: f64) -> Vec<u8> {
        Self {
            amount: Decimal::from(amount),
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn getAccountsWasm(
        programKey: &[u8],
        userKey: &[u8],
        collateralMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let userKey = UserKey::new(b2pk(userKey)?);
        let collateralMintKey = CollateralMintKey::new(b2pk(collateralMintKey)?);
        let accounts = Self::get_accounts(programKey, (userKey, collateralMintKey))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VaultDeposit {}

impl Command for VaultDeposit {
    const ID: u32 = 0x295bcc0f;
    type Keys = (UserKey, CollateralMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, collateral_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *user_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_vault(&user_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_collateral(&collateral_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: user_key.derive_associated_token_address(&collateral_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_safe(&collateral_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: spl_token::id(),
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let vault_account = Writable::new(&accounts[1]);
        let collateral_account = Writable::new(&accounts[2]);
        let user_token_account = TokenAccount::new(Writable::new(&accounts[3]));
        let collateral_token_account = TokenAccount::new(Writable::new(&accounts[4]));
        let token_program_account = TokenProgramAccount::new(&accounts[5]);

        let mut vault_data = vault_account.get_info().data.borrow_mut();
        let (vault, vault_auth) =
            Vault::load_auth(program_id, vault_account, &mut vault_data[..], user_account);

        let mut collateral_data = collateral_account.get_info().data.borrow_mut();
        let collateral =
            Collateral::load_mut(program_id, collateral_account, &mut collateral_data[..], ());

        vault.deposit(
            vault_auth,
            self.amount,
            program_id,
            collateral,
            user_account,
            user_token_account,
            collateral_token_account,
            token_program_account,
        );
    }
}
