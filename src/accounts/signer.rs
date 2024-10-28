use {crate::{traits::Account, util::revert}, solana_program::account_info::AccountInfo};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Signer(&'static AccountInfo<'static>);

impl Signer {
    pub const fn new(info: &'static AccountInfo<'static>) -> Self {
        if !info.is_signer {
            revert("signer account must be a signer");
        }
        Self(info)
    }
}

impl Account for Signer {
    fn get_info(self) -> &'static AccountInfo<'static> {
        &self.0
    }
}
