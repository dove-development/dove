#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::Writable,
        keys::VaultKey,
        store::Vault,
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};

/// Unliquidates a vault
///
/// Accounts expected:
/// 0. `[writable]` Vault account (PDA) to be unliquidated
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct VaultUnliquidate {
    _private: (),
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl VaultUnliquidate {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm() -> Vec<u8> {
        Self { _private: () }.get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(vaultKey: &[u8]) -> Result<Vec<AccountWasm>, String> {
        let vaultKey = VaultKey::new(b2pk(vaultKey)?);
        let accounts = Self::get_accounts(ProgramKey::new(Pubkey::default()), (vaultKey,))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for VaultUnliquidate {}

impl Command for VaultUnliquidate {
    const ID: u32 = 0xd80a50bf;
    type Keys = (VaultKey,);

    #[cfg(feature = "wasm")]
    fn get_accounts(_: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (vault_key,) = keys;
        vec![AccountMeta {
            pubkey: *vault_key,
            is_signer: false,
            is_writable: true,
        }]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let vault_account = Writable::new(&accounts[0]);

        let mut vault_data = vault_account.get_info().data.borrow_mut();
        let vault = Vault::load_mut(program_id, vault_account, &mut vault_data[..], ());

        vault.unliquidate();
    }
}
