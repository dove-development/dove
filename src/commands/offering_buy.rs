#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
        finance::Decimal,
        keys::{DoveMintKey, DvdMintKey, UserKey},
        store::{Authority, World},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Buys from the current world offering
///
/// Accounts expected:
///
/// 0. `[signer]` User account (buyer)
/// 1. `[writable]` World account (PDA)
/// 2. `[writable]` Debt token mint account
/// 3. `[writable]` User's DVD account
/// 4. `[writable]` Equity token mint account
/// 5. `[writable]` User's DOVE account
/// 6. `[]` Authority account (PDA)
/// 7. `[]` SPL Token program
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct OfferingBuy {
    requested_base_amount: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl OfferingBuy {
    #[wasm_bindgen(js_name = "getData")]
    #[allow(non_snake_case)]
    pub fn get_data_wasm(requestedBaseAmount: f64) -> Vec<u8> {
        Self {
            requested_base_amount: Decimal::from(requestedBaseAmount),
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        dvdMintKey: &[u8],
        doveMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let userKey = UserKey::new(b2pk(userKey)?);
        let dvdMintKey = DvdMintKey::new(b2pk(dvdMintKey)?);
        let doveMintKey = DoveMintKey::new(b2pk(doveMintKey)?);
        let accounts = Self::get_accounts(programKey, (userKey, dvdMintKey, doveMintKey))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for OfferingBuy {}

impl Command for OfferingBuy {
    const ID: u32 = 0x1bf0be84;
    type Keys = (UserKey, DvdMintKey, DoveMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, dvd_mint_key, dove_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *user_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *dvd_mint_key,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: user_key.derive_associated_token_address(&dvd_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *dove_mint_key,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: user_key.derive_associated_token_address(&dove_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_authority(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: spl_token::ID,
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let world_account = Writable::new(&accounts[1]);
        let dvd_mint_account = MintAccount::new(Writable::new(&accounts[2]));
        let dvd_account = TokenAccount::new(Writable::new(&accounts[3]));
        let dove_mint_account = MintAccount::new(Writable::new(&accounts[4]));
        let dove_token_account = TokenAccount::new(Writable::new(&accounts[5]));
        let authority_account = Readonly::new(&accounts[6]);
        let token_program_account = TokenProgramAccount::new(&accounts[7]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let authority = Authority::from_account(program_id, authority_account);

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        world.offering.buy(
            self.requested_base_amount,
            &clock,
            &mut world.dvd,
            &mut world.dove,
            authority,
            world.config.get_auction_config(),
            user_account,
            dvd_mint_account,
            dvd_account,
            dove_mint_account,
            dove_token_account,
            token_program_account,
        );
    }
}
