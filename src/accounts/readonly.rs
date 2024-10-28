use {crate::{traits::Account, util::revert}, solana_program::account_info::AccountInfo};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Readonly(&'static AccountInfo<'static>);

impl Readonly {
    pub const fn new(info: &'static AccountInfo<'static>) -> Self {
        if info.is_writable {
            revert("readonly account should not be writable");
        }
        if info.is_signer {
            revert("readonly account should not be a signer");
        }
        Self(info)
    }
}

impl Account for Readonly {
    fn get_info(self) -> &'static AccountInfo<'static> {
        &self.0
    }
}
