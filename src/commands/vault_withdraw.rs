#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::{prelude::wasm_bindgen, JsValue},
};
use {
    crate::{
        accounts::{Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::Decimal,
        keys::{CollateralMintKey, OracleKey, UserKey},
        store::{Authority, Collateral, Vault, World},
        traits::{Account, Command, Pod, Store},
        util::revert,
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Withdraws tokens from a vault
///
/// Accounts expected:
/// 0. `[signer]` User account
/// 1. `[writable]` Vault account (PDA)
/// 2. `[writable]` World account (PDA)
/// 3. `[writable]` User's token account (destination for tokens)
/// 4. `[writable]` Safe account (source of tokens)
/// 5. `[]` SPL Token program
/// 6. `[]` Authority account (PDA)
/// 7..n. `[writable]` Collateral accounts for reserves in the vault, in order (PDAs)
/// n..m. `[]` Oracle accounts for reserves in the vault, in order
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VaultWithdraw {
    requested_amount: Decimal,
    reserve_index: u8,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VaultWithdraw {
    #[wasm_bindgen(js_name = "getData")]
    #[allow(non_snake_case)]
    pub fn get_data_wasm(requestedAmount: f64, reserveIndex: u8) -> Vec<u8> {
        Self {
            requested_amount: Decimal::from(requestedAmount),
            reserve_index: reserveIndex,
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        collateralMintKeys: Vec<JsValue>,
        oracleKeys: Vec<JsValue>,
        reserveIndex: u8,
    ) -> Result<Vec<AccountWasm>, String> {
        let program_key = ProgramKey::new(b2pk(programKey)?);
        let user_key = UserKey::new(b2pk(userKey)?);
        let collateral_mint_keys = collateralMintKeys
            .into_iter()
            .map(|key| -> Result<CollateralMintKey, String> {
                let key_bytes: [u8; 32] = serde_wasm_bindgen::from_value(key)
                    .map_err(|e| format!("Invalid collateral mint key: {}", e))?;
                Ok(CollateralMintKey::new(b2pk(&key_bytes)?))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let oracle_keys = oracleKeys
            .into_iter()
            .map(|key| -> Result<OracleKey, String> {
                let key_bytes: [u8; 32] = serde_wasm_bindgen::from_value(key)
                    .map_err(|e| format!("Invalid oracle key: {}", e))?;
                Ok(OracleKey::new(b2pk(&key_bytes)?))
            })
            .collect::<Result<Vec<_>, String>>()?;

        let accounts = Self::get_accounts(
            program_key,
            (user_key, collateral_mint_keys, oracle_keys, reserveIndex),
        )
        .into_iter()
        .map(AccountWasm::from)
        .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VaultWithdraw {}

impl Command for VaultWithdraw {
    const ID: u32 = 0x4d771aa0;
    type Keys = (UserKey, Vec<CollateralMintKey>, Vec<OracleKey>, u8);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, collateral_mint_keys, oracle_keys, reserve_index) = keys;
        let mut v = vec![
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
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: user_key
                    .derive_associated_token_address(&collateral_mint_keys[reserve_index as usize]),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_safe(&collateral_mint_keys[reserve_index as usize]),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: spl_token::id(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_authority(),
                is_signer: false,
                is_writable: false,
            },
        ];
        v.extend(collateral_mint_keys.into_iter().map(|key| AccountMeta {
            pubkey: program_key.derive_collateral(&key),
            is_signer: false,
            is_writable: true,
        }));
        v.extend(oracle_keys.into_iter().map(|key| AccountMeta {
            pubkey: *key,
            is_signer: false,
            is_writable: false,
        }));
        v
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let vault_account = Writable::new(&accounts[1]);
        let world_account = Writable::new(&accounts[2]);
        let destination_token_account = TokenAccount::new(Writable::new(&accounts[3]));
        let program_token_account = TokenAccount::new(Writable::new(&accounts[4]));
        let token_program_account = TokenProgramAccount::new(&accounts[5]);
        let authority_account = Readonly::new(&accounts[6]);
        let (collateral_accounts, oracle_accounts) = {
            let a = &accounts[7..];
            if (a.len() % 2) != 0 {
                revert("there should be an equal # of collateral and oracle accounts");
            }
            a.split_at(a.len() / 2)
        };
        let collateral_accounts = collateral_accounts
            .iter()
            .map(Writable::new)
            .collect::<Vec<_>>();
        let oracle_accounts = oracle_accounts
            .iter()
            .map(Readonly::new)
            .collect::<Vec<_>>();

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let mut vault_data = vault_account.get_info().data.borrow_mut();
        let (vault, vault_auth) =
            Vault::load_auth(program_id, vault_account, &mut vault_data[..], user_account);

        let authority = Authority::from_account(program_id, authority_account);

        let mut collateral_data: Vec<_> = collateral_accounts
            .iter()
            .map(|account| account.get_info().data.borrow_mut())
            .collect();
        let collateral = collateral_accounts
            .into_iter()
            .zip(&mut collateral_data)
            .map(|(acc, data)| Collateral::load_mut(program_id, acc, data, ()))
            .collect::<Vec<_>>();

        let clock = Clock::get().map_err(|_| "could not get clock").unwrap();
        vault.withdraw(
            vault_auth,
            self.requested_amount,
            &mut world.debt,
            &world.config.get_debt_config(),
            world.config.get_max_ltv(),
            program_id,
            program_token_account,
            destination_token_account,
            token_program_account,
            authority,
            collateral,
            &oracle_accounts,
            self.reserve_index as usize,
            &clock,
        );
    }
}
