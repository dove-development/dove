#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{
            MintAccount, Readonly, Signer, SystemProgramAccount, TokenProgramAccount, Writable,
        },
        keys::{CollateralMintKey, SovereignKey},
        store::{Authority, Collateral, CollateralParams, World},
        token::Safe,
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey, rent::Rent, sysvar::Sysvar},
};

/// Creates a new collateral type in the system
///
/// Accounts expected:
///
/// 0. `[signer]` Sovereign account (paying for account creation)
/// 1. `[writable]` Collateral account (PDA, will be created)
/// 2. `[writable]` Safe account (PDA, will be created)
/// 3. `[]` Authority account (PDA)
/// 4. `[]` World account (PDA)
/// 5. `[]` Mint account (for the collateral token)
/// 6. `[]` System program
/// 7. `[]` SPL Token program
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct CollateralCreate {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl CollateralCreate {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        sovereignKey: &[u8],
        collateralMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let sovereignKey = SovereignKey::new(b2pk(sovereignKey)?);
        let collateralMintKey = CollateralMintKey::new(b2pk(collateralMintKey)?);
        let accounts = Self::get_accounts(programKey, (sovereignKey, collateralMintKey))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for CollateralCreate {}

impl Command for CollateralCreate {
    const ID: u32 = 0xe20af14f;
    type Keys = (SovereignKey, CollateralMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (sovereign_key, collateral_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *sovereign_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_collateral(&collateral_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_safe(&collateral_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_authority(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: *collateral_mint_key,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: solana_program::system_program::ID,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: spl_token::ID,
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let sovereign_account = Signer::new(&accounts[0]);
        let collateral_account = Writable::new(&accounts[1]);
        let safe_account = Writable::new(&accounts[2]);
        let authority_account = Readonly::new(&accounts[3]);
        let world_account = Readonly::new(&accounts[4]);
        let mint_account = MintAccount::new(Readonly::new(&accounts[5]));
        let system_program_account = SystemProgramAccount::new(&accounts[6]);
        let token_program_account = TokenProgramAccount::new(&accounts[7]);

        let world_data = world_account.get_info().data.borrow();
        let world = World::load(program_id, world_account, &world_data, ());
        let sovereign_auth = world.sovereign.authorize(sovereign_account);

        let authority = Authority::from_account(program_id, authority_account);
        let rent = Rent::get().map_err(|_| "Failed to get rent").unwrap();

        let safe_account = Safe::create(
            program_id,
            sovereign_account,
            safe_account,
            mint_account,
            system_program_account,
            token_program_account,
            authority,
            &rent,
            sovereign_auth,
        );

        Collateral::create(
            program_id,
            sovereign_account,
            collateral_account,
            system_program_account,
            mint_account,
            &rent,
            CollateralParams {
                sovereign_auth,
                safe_nonce: safe_account.get_nonce(),
                mint_account,
            },
        );
    }
}
