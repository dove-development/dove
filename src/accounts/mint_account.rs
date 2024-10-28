use {
    crate::{traits::Account, util::require},
    solana_program::{account_info::AccountInfo, program_pack::Pack},
    spl_token::state::Mint as SplMint,
};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MintAccount<T: Account>(T);

impl<T: Account + Copy> MintAccount<T> {
    pub fn new(account: T) -> Self {
        let info = account.get_info();
        require(
            info.owner == &spl_token::ID,
            "mint account should be owned by spl token program",
        );
        require(
            info.data_len() == SplMint::LEN,
            "mint account should be of length Mint::LEN",
        );        
        Self(account)
    }
}

impl<T: Account> Account for MintAccount<T> {
    fn get_info(self) -> &'static AccountInfo<'static> {
        self.0.get_info()
    }
}
