#[cfg(feature = "wasm")]
use crate::keys::ProgramKey;
#[cfg(feature = "wasm")]
use solana_program::instruction::AccountMeta;
use {
    crate::traits::Pod,
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};

pub trait Command: Pod {
    const ID: u32;
    type Keys;
    #[allow(unused)]
    fn execute(
        self,
        program_id: &'static Pubkey,
        accounts: &'static [AccountInfo],
    );
    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta>;
    #[cfg(feature = "wasm")]
    fn get_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(4 + Self::SIZE);
        data.extend_from_slice(&Self::ID.to_le_bytes());
        data.extend_from_slice(self.as_bytes());
        data
    }
}
