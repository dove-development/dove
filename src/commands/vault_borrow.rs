#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::{wasm_bindgen, JsValue},
};
use {
    crate::{
        accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::Decimal,
        keys::{CollateralMintKey, DvdMintKey, OracleKey, UserKey},
        store::{Authority, Collateral, Vault, World},
        traits::{Account, Command, Pod, Store},
        util::revert,
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Borrows tokens from the world
///
/// Accounts expected:
///
/// 0. `[signer]` User account
/// 1. `[writable]` Mint account (for the DVD)
/// 2. `[writable]` Debt token account (to receive borrowed tokens)
/// 3. `[writable]` World account (PDA)
/// 4. `[writable]` Vault account (PDA)
/// 5. `[]` Authority account (PDA)
/// 6. `[]` SPL Token program
/// 7..n. `[]` Collateral accounts in order of vault reserves (PDAs)
/// n..m. `[]` Oracle accounts in order of vault reserves (PDAs)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VaultBorrow {
    requested_amount: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VaultBorrow {
    #[wasm_bindgen(js_name = "getData")]
    #[allow(non_snake_case)]
    pub fn get_data_wasm(requestedAmount: f64) -> Vec<u8> {
        Self {
            requested_amount: Decimal::from(requestedAmount),
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        dvdMintKey: &[u8],
        collateralMintKeys: Vec<JsValue>,
        oracleKeys: Vec<JsValue>,
    ) -> Result<Vec<AccountWasm>, String> {
        let program_key = ProgramKey::new(b2pk(programKey)?);
        let user_key = UserKey::new(b2pk(userKey)?);
        let dvd_mint_key = DvdMintKey::new(b2pk(dvdMintKey)?);
        let collateral_mint_keys = collateralMintKeys
            .into_iter()
            .map(|key| -> Result<CollateralMintKey, String> {
                let key_bytes: [u8; 32] = serde_wasm_bindgen::from_value(key)
                    .map_err(|e| format!("invalid collateral mint key: {}", e))?;
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
            (user_key, dvd_mint_key, collateral_mint_keys, oracle_keys),
        )
        .into_iter()
        .map(AccountWasm::from)
        .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VaultBorrow {}

impl Command for VaultBorrow {
    const ID: u32 = 0x0b05f1e1;
    type Keys = (UserKey, DvdMintKey, Vec<CollateralMintKey>, Vec<OracleKey>);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, dvd_mint_key, collateral_mint_keys, oracle_keys) = keys;
        let mut v = vec![
            AccountMeta {
                pubkey: *user_key,
                is_signer: true,
                is_writable: false,
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
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_vault(&user_key),
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
        ];
        v.extend(collateral_mint_keys.into_iter().map(|c| AccountMeta {
            pubkey: program_key.derive_collateral(&c),
            is_signer: false,
            is_writable: false,
        }));
        v.extend(oracle_keys.into_iter().map(|o| AccountMeta {
            pubkey: *o,
            is_signer: false,
            is_writable: false,
        }));
        v
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let mint_account = MintAccount::new(Writable::new(&accounts[1]));
        let dvd_account = TokenAccount::new(Writable::new(&accounts[2]));
        let world_account = Writable::new(&accounts[3]);
        let vault_account = Writable::new(&accounts[4]);
        let authority_account = Readonly::new(&accounts[5]);
        let token_program_account = TokenProgramAccount::new(&accounts[6]);
        let (collateral_accounts, oracle_accounts) = {
            let a = &accounts[7..];
            if (a.len() % 2) != 0 {
                revert("there should be an equal # of collateral and oracle accounts");
            }
            a.split_at(a.len() / 2)
        };
        let oracle_accounts = oracle_accounts
            .iter()
            .map(Readonly::new)
            .collect::<Vec<_>>();
        let collateral_accounts = collateral_accounts
            .iter()
            .map(Readonly::new)
            .collect::<Vec<_>>();
        let collateral_data = collateral_accounts
            .iter()
            .map(|c| c.get_info().data.borrow())
            .collect::<Vec<_>>();
        let collateral = collateral_accounts
            .into_iter()
            .zip(&collateral_data)
            .map(|(acc, data)| Collateral::load(program_id, acc, data, ()))
            .collect::<Vec<_>>();

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let mut vault_data = vault_account.get_info().data.borrow_mut();
        let (vault, vault_auth) =
            Vault::load_auth(program_id, vault_account, &mut vault_data[..], user_account);

        let authority = Authority::from_account(program_id, authority_account);

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        vault.borrow(
            vault_auth,
            self.requested_amount,
            &mut world.debt,
            &world.config.get_debt_config(),
            &mut world.dvd,
            &mut world.dvd_price,
            &world.config.get_dvd_interest_rate(),
            world.config.get_max_ltv(),
            authority,
            &collateral,
            &oracle_accounts,
            mint_account,
            dvd_account,
            token_program_account,
            &clock,
        );
    }
}
