use {
    super::Readonly,
    crate::{traits::Account, util::require},
    solana_program::{account_info::AccountInfo, system_program},
};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct SystemProgramAccount(Readonly);

impl SystemProgramAccount {
    pub fn new(info: &'static AccountInfo<'static>) -> Self {
        require(
            info.key == &system_program::ID,
            "Invalid system program account",
        );
        Self(Readonly::new(info))
    }
}

impl Account for SystemProgramAccount {
    fn get_info(self) -> &'static AccountInfo<'static> {
        self.0.get_info()
    }
}
