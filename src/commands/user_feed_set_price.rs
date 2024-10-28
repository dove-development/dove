#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{Signer, Writable},
        finance::Decimal,
        keys::UserKey,
        oracle::UserFeed,
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, pubkey::Pubkey},
};

/// Sets the price for a user feed
///
/// Accounts expected:
///
/// 0. `[signer]` User account (owner of the UserFeed)
/// 1. `[writable]` UserFeed account (PDA)
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct UserFeedSetPrice {
    price: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl UserFeedSetPrice {
    #[wasm_bindgen(js_name = "getData")]
    pub fn get_data_wasm(price: f64) -> Vec<u8> {
        Self {
            price: Decimal::from(price),
        }
        .get_data()
    }
    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        index: u8,
    ) -> Result<Vec<AccountWasm>, String> {
        let program_key = ProgramKey::new(b2pk(programKey)?);
        let user_key = UserKey::new(b2pk(userKey)?);
        let accounts = Self::get_accounts(program_key, (user_key, index))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for UserFeedSetPrice {}

impl Command for UserFeedSetPrice {
    const ID: u32 = 0x9fe4e061;
    type Keys = (UserKey, u8);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, (user_key, index): Self::Keys) -> Vec<AccountMeta> {
        vec![
            AccountMeta {
                pubkey: *user_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_user_feed(&user_key, index),
                is_signer: false,
                is_writable: true,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let user_feed_account = Writable::new(&accounts[1]);

        let mut user_feed_data = user_feed_account.get_info().data.borrow_mut();
        let (user_feed, user_feed_auth) = UserFeed::load_auth(
            program_id,
            user_feed_account,
            &mut user_feed_data[..],
            user_account,
        );

        user_feed.set_price(user_feed_auth, self.price);
    }
}
