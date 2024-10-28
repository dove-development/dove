use solana_program::account_info::AccountInfo;

pub trait Account: Sized {
    fn get_info(self) -> &'static AccountInfo<'static>;
}
