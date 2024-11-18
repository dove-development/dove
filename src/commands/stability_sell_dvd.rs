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
        keys::{DvdMintKey, StableMintKey, UserKey},
        store::{Authority, Stability, World},
        traits::{Account, Command, Pod, Store},
    },
    solana_program::{account_info::AccountInfo, clock::Clock, pubkey::Pubkey, sysvar::Sysvar},
};

/// Sells DVD to the stability pool in exchange for stable tokens
///
/// Accounts expected:
///
/// 0. `[signer]` User account (seller)
/// 1. `[writable]` DVD source token account (to sell DVD from)
/// 2. `[writable]` DVD token mint account
/// 3. `[writable]` Safe account (to take stable tokens from)
/// 4. `[writable]` Stable token destination account (to receive stable tokens)
/// 5. `[writable]` World account (PDA)
/// 6. `[writable]` Stability account (PDA)
/// 7. `[]` Authority account (PDA)
/// 8. `[]` SPL Token program
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct StabilitySellDvd {
    amount: Decimal,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl StabilitySellDvd {
    #[wasm_bindgen(js_name = "getData")]
    #[allow(non_snake_case)]
    pub fn get_data_wasm(amount: f64) -> Vec<u8> {
        Self {
            amount: Decimal::from(amount),
        }
        .get_data()
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        userKey: &[u8],
        dvdMintKey: &[u8],
        stableMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let programKey = ProgramKey::new(b2pk(programKey)?);
        let userKey = UserKey::new(b2pk(userKey)?);
        let dvdMintKey = DvdMintKey::new(b2pk(dvdMintKey)?);
        let stableMintKey = StableMintKey::new(b2pk(stableMintKey)?);
        let accounts = Self::get_accounts(programKey, (userKey, dvdMintKey, stableMintKey))
            .into_iter()
            .map(AccountWasm::from)
            .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for StabilitySellDvd {}

impl Command for StabilitySellDvd {
    const ID: u32 = 0x31cac1de;
    type Keys = (UserKey, DvdMintKey, StableMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (user_key, dvd_mint_key, stable_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *user_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: user_key.derive_associated_token_address(&dvd_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: *dvd_mint_key,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_safe(&stable_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: user_key.derive_associated_token_address(&stable_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_stability(&stable_mint_key),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_authority(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: spl_token::id(),
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let user_account = Signer::new(&accounts[0]);
        let dvd_source_token_account = TokenAccount::new(Writable::new(&accounts[1]));
        let dvd_mint_account = MintAccount::new(Writable::new(&accounts[2]));
        let safe_account = TokenAccount::new(Writable::new(&accounts[3]));
        let stable_destination_token_account = TokenAccount::new(Writable::new(&accounts[4]));
        let world_account = Writable::new(&accounts[5]);
        let stability_account = Writable::new(&accounts[6]);
        let authority_account = Readonly::new(&accounts[7]);
        let token_program_account = TokenProgramAccount::new(&accounts[8]);

        let mut world_data = world_account.get_info().data.borrow_mut();
        let world = World::load_mut(program_id, world_account, &mut world_data[..], ());

        let mut stability_data = stability_account.get_info().data.borrow_mut();
        let stability =
            Stability::load_mut(program_id, stability_account, &mut stability_data[..], ());

        let authority = Authority::from_account(program_id, authority_account);

        let clock = Clock::get().unwrap();
        stability.sell_dvd(
            self.amount,
            &mut world.dvd,
            &mut world.dvd_price,
            &mut world.config.get_dvd_interest_rate(),
            &mut world.stable_dvd,
            program_id,
            authority,
            user_account,
            safe_account,
            stable_destination_token_account,
            dvd_source_token_account,
            dvd_mint_account,
            token_program_account,
            &clock,
        );
    }
}
