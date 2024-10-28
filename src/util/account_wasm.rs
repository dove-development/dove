use {
    solana_program::{instruction::AccountMeta, pubkey::Pubkey},
    wasm_bindgen::prelude::wasm_bindgen,
};

#[wasm_bindgen]
pub struct AccountWasm {
    pubkey: Pubkey,
    is_signer: bool,
    is_writable: bool,
}

#[wasm_bindgen]
impl AccountWasm {
    pub fn get_key(&self) -> Vec<u8> {
        self.pubkey.to_bytes().to_vec()
    }

    pub fn is_signer(&self) -> bool {
        self.is_signer
    }

    pub fn is_writable(&self) -> bool {
        self.is_writable
    }
}

impl From<AccountMeta> for AccountWasm {
    fn from(value: AccountMeta) -> Self {
        Self {
            pubkey: value.pubkey,
            is_signer: value.is_signer,
            is_writable: value.is_writable,
        }
    }
}
