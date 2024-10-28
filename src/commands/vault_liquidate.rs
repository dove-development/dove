#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::{wasm_bindgen, JsValue},
};
use {
    crate::{
        accounts::{MintAccount, Readonly, TokenAccount, TokenProgramAccount, Writable},
        keys::{CollateralMintKey, DvdMintKey, OracleKey, UserKey, VaultKey},
        store::{Authority, Collateral, Vault, World},
        traits::{Account, Command, Pod, Store},
        util::revert,
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Liquidates a vault
///
/// Accounts expected:
///
/// 0. `[writable]` Debt token mint account
/// 1. `[writable]` Debt token account (to receive liquidation reward)
/// 2. `[writable]` World account (PDA)
/// 3. `[writable]` Vault account (PDA) to be liquidated
/// 4. `[]` Authority account (PDA)
/// 5. `[]` SPL Token program
/// 6..n. `[]` Collateral accounts in order of vault reserves (PDAs)
/// n..m. `[]` Oracle accounts in order of vault reserves (PDAs)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VaultLiquidate {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VaultLiquidate {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn getAccountsWasm(
        programKey: &[u8],
        userKey: &[u8],
        vaultKey: &[u8],
        dvdMintKey: &[u8],
        collateralMintKeys: Vec<JsValue>,
        oracleKeys: Vec<JsValue>,
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let userKey = UserKey::new(b2pk(userKey)?);
        let vaultKey = VaultKey::new(b2pk(vaultKey)?);
        let dvdMintKey = DvdMintKey::new(b2pk(dvdMintKey)?);
        let collateralMintKeys = collateralMintKeys
            .into_iter()
            .map(|key| -> Result<CollateralMintKey, String> {
                let keyBytes: [u8; 32] = serde_wasm_bindgen::from_value(key)
                    .map_err(|e| format!("Invalid collateral mint key: {}", e))?;
                Ok(CollateralMintKey::new(b2pk(&keyBytes)?))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let oracleKeys = oracleKeys
            .into_iter()
            .map(|key| -> Result<OracleKey, String> {
                let keyBytes: [u8; 32] = serde_wasm_bindgen::from_value(key)
                    .map_err(|e| format!("Invalid oracle key: {}", e))?;
                Ok(OracleKey::new(b2pk(&keyBytes)?))
            })
            .collect::<Result<Vec<_>, String>>()?;

        let accounts = Self::get_accounts(
            programKey,
            (
                userKey,
                vaultKey,
                dvdMintKey,
                collateralMintKeys,
                oracleKeys,
            ),
        )
        .into_iter()
        .map(AccountWasm::from)
        .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VaultLiquidate {}

impl Command for VaultLiquidate {
    const ID: u32 = 0x963d62f0;
    type Keys = (
        UserKey,
        VaultKey,
        DvdMintKey,
        Vec<CollateralMintKey>,
        Vec<OracleKey>,
    );

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, vault_key, dvd_mint_key, collateral_mint_keys, oracle_keys) = keys;
        let mut v = vec![
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
        let dvd_mint_account = MintAccount::new(Writable::new(&accounts[0]));
        let dvd_account = TokenAccount::new(Writable::new(&accounts[1]));
        let world_account = Writable::new(&accounts[2]);
        let vault_account = Writable::new(&accounts[3]);
        let authority_account = Readonly::new(&accounts[4]);
        let token_program_account = TokenProgramAccount::new(&accounts[5]);
        let (collateral_accounts, oracle_accounts) = {
            let a = &accounts[6..];
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
        let vault = Vault::load_mut(program_id, vault_account, &mut vault_data[..], ());

        let authority = Authority::from_account(program_id, authority_account);
        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        vault.liquidate(
            world.config.get_max_ltv(),
            &mut world.debt,
            &world.config.get_debt_config(),
            &world.config.get_vault_config(),
            &mut world.dvd,
            &collateral,
            &oracle_accounts,
            dvd_mint_account,
            dvd_account,
            token_program_account,
            authority,
            &clock,
        );
    }
}
