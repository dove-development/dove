use {
    super::Readonly,
    crate::{traits::Account, util::require},
    solana_program::{account_info::AccountInfo, sysvar::instructions},
};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct InstructionSysvarAccount(Readonly);

impl InstructionSysvarAccount {
    pub fn new(info: &'static AccountInfo<'static>) -> Self {
        require(
            info.key == &instructions::ID,
            "Invalid instruction sysvar account",
        );
        Self(Readonly::new(info))
    }
}

impl Account for InstructionSysvarAccount {
    fn get_info(self) -> &'static AccountInfo<'static> {
        self.0.get_info()
    }
}
