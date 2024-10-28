use {
    crate::{traits::Account, util::require},
    solana_program::{account_info::AccountInfo, program_pack::Pack},
    spl_token::state::Account as TokenAccountState,
};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TokenAccount<T: Account>(T);

impl<T: Account + Copy> TokenAccount<T> {
    pub fn new(account: T) -> Self {
        let info = account.get_info();
        require(info.owner == &spl_token::ID, "Invalid token account");
        require(
            info.data_len() == TokenAccountState::LEN,
            "Invalid token account length",
        );
        Self(account)
    }
}

impl<T: Account> Account for TokenAccount<T> {
    fn get_info(self) -> &'static AccountInfo<'static> {
        self.0.get_info()
    }
}
