#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::Decimal,
        keys::{CollateralMintKey, DvdMintKey, UserKey, VaultKey},
        store::{Authority, Collateral, Vault, World},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Buys collateral from a liquidated vault
///
/// Accounts expected:
///
/// 0. `[signer]` User account (buyer)
/// 1. `[writable]` DVD token account (to pay for collateral)
/// 2. `[writable]` DVD token mint account
/// 3. `[writable]` Safe account (to take bought collateral from)
/// 4. `[writable]` Collateral destination token account (to receive bought collateral)
/// 5. `[writable]` World account (PDA)
/// 6. `[writable]` Vault account (PDA)
/// 7. `[]` Authority account (PDA)
/// 8. `[]` SPL Token program
/// 9. `[writable]` Collateral account (PDA)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VaultBuyCollateral {
    requested_dvd_amount: Decimal,
    collateral_index: u8,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VaultBuyCollateral {
    #[wasm_bindgen(js_name = "getData")]
    #[allow(non_snake_case)]
    pub fn get_data_wasm(requestedDvdAmount: f64, collateralIndex: u8) -> Vec<u8> {
        Self {
            requested_dvd_amount: Decimal::from(requestedDvdAmount),
            collateral_index: collateralIndex,
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        vaultKey: &[u8],
        dvdMintKey: &[u8],
        collateralMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let userKey = UserKey::new(b2pk(userKey)?);
        let vaultKey = VaultKey::new(b2pk(vaultKey)?);
        let dvdMintKey = DvdMintKey::new(b2pk(dvdMintKey)?);
        let collateralMintKey = CollateralMintKey::new(b2pk(collateralMintKey)?);
        let accounts = Self::get_accounts(
            programKey,
            (userKey, vaultKey, dvdMintKey, collateralMintKey),
        )
        .into_iter()
        .map(AccountWasm::from)
        .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VaultBuyCollateral {}

impl Command for VaultBuyCollateral {
    const ID: u32 = 0xb91a7697;
    type Keys = (UserKey, VaultKey, DvdMintKey, CollateralMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, vault_key, dvd_mint_key, collateral_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *user_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: user_key.derive_associated_token_address(&dvd_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *dvd_mint_key,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_safe(&collateral_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: user_key.derive_associated_token_address(&collateral_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *vault_key,
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
                pubkey: program_key.derive_collateral(&collateral_mint_key),
                is_signer: false,
                is_writable: true,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let dvd_token_account = TokenAccount::new(Writable::new(&accounts[1]));
        let dvd_mint_account = MintAccount::new(Writable::new(&accounts[2]));
        let safe_account = TokenAccount::new(Writable::new(&accounts[3]));
        let collateral_destination_token_account = TokenAccount::new(Writable::new(&accounts[4]));
        let world_account = Writable::new(&accounts[5]);
        let vault_account = Writable::new(&accounts[6]);
        let authority_account = Readonly::new(&accounts[7]);
        let token_program_account = TokenProgramAccount::new(&accounts[8]);
        let collateral_account = Writable::new(&accounts[9]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let mut vault_data = vault_account.get_info().data.borrow_mut();
        let vault = Vault::load_mut(program_id, vault_account, &mut vault_data[..], ());

        let mut collateral_data = collateral_account.get_info().data.borrow_mut();
        let collateral =
            Collateral::load_mut(program_id, collateral_account, &mut collateral_data[..], ());

        let authority = Authority::from_account(program_id, authority_account);

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        vault.buy_collateral(
            self.requested_dvd_amount,
            program_id,
            collateral,
            &mut world.debt,
            &world.config.get_debt_config(),
            &mut world.dvd,
            &world.config.get_auction_config(),
            user_account,
            dvd_token_account,
            dvd_mint_account,
            safe_account,
            collateral_destination_token_account,
            token_program_account,
            self.collateral_index as usize,
            authority,
            &clock,
        );
    }
}
