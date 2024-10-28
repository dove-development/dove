use {
    super::Readonly,
    crate::{traits::Account, util::require},
    solana_program::account_info::AccountInfo,
};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TokenProgramAccount(Readonly);

impl TokenProgramAccount {
    pub fn new(info: &'static AccountInfo<'static>) -> Self {
        require(info.key == &spl_token::ID, "Invalid token program account");
        Self(Readonly::new(info))
    }
}

impl Account for TokenProgramAccount {
    fn get_info(self) -> &'static AccountInfo<'static> {
        self.0.get_info()
    }
}
