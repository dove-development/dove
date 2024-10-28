use {crate::{traits::Account, util::revert}, solana_program::account_info::AccountInfo};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Writable(&'static AccountInfo<'static>);

impl Writable {
    pub const fn new(info: &'static AccountInfo<'static>) -> Self {
        if !info.is_writable {
            revert("writable account should be writable");
        }
        Self(info)
    }
}

impl Account for Writable {
    fn get_info(self) -> &'static AccountInfo<'static> {
        &self.0
    }
}
