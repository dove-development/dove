#[cfg(feature = "wasm")]
use {
    crate::keys::ProgramKey,
    crate::util::{b2pk, AccountWasm},
    solana_program::instruction::AccountMeta,
    wasm_bindgen::prelude::wasm_bindgen,
};
use {
    crate::{
        accounts::{MintAccount, Readonly, Signer, SystemProgramAccount, Writable},
        finance::Schedule,
        keys::{DoveMintKey, DvdMintKey, SovereignKey},
        store::{Authority, World, WorldParams},
        traits::{Command, Pod, Store},
    },
    solana_program::{
        account_info::AccountInfo, clock::Clock, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
    },
};

/// Creates the program's world account
///
/// Accounts expected:
///
/// 0. `[signer]` Sovereign account
/// 1. `[writable]` World account (PDA, will be created)
/// 2. `[]` Authority account (PDA)
/// 3. `[]` Mint account for the DVD
/// 4. `[]` Mint account for the DOVE
/// 5. `[]` System program
#[repr(C)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct WorldCreate {
    vesting_recipient: Pubkey,
    vesting_schedule: Schedule,
}
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WorldCreate {
    #[wasm_bindgen(js_name = "getData")]
    #[allow(non_snake_case)]
    pub fn get_data_wasm(
        vestingRecipient: Vec<u8>,
        vestingSchedule: Schedule,
    ) -> Result<Vec<u8>, String> {
        Ok(Self {
            vesting_recipient: Pubkey::new_from_array(
                vestingRecipient
                    .try_into()
                    .map_err(|_| "Invalid vesting recipient")?,
            ),
            vesting_schedule: vestingSchedule,
        }
        .get_data())
    }

    #[wasm_bindgen(js_name = "getAccounts")]
    #[allow(non_snake_case)]
    pub fn get_accounts_wasm(
        programKey: &[u8],
        sovereignKey: &[u8],
        dvdMintKey: &[u8],
        doveMintKey: &[u8],
    ) -> Result<Vec<AccountWasm>, String> {
        let program_key = ProgramKey::new(b2pk(programKey)?);
        let sovereign_key = SovereignKey::new(b2pk(sovereignKey)?);
        let dvd_mint_key = DvdMintKey::new(b2pk(dvdMintKey)?);
        let dove_mint_key = DoveMintKey::new(b2pk(doveMintKey)?);
        let accounts =
            Self::get_accounts(program_key, (sovereign_key, dvd_mint_key, dove_mint_key))
                .into_iter()
                .map(AccountWasm::from)
                .collect();
        Ok(accounts)
    }
}

unsafe impl Pod for WorldCreate {}

impl Command for WorldCreate {
    const ID: u32 = 0xf50eb479;
    type Keys = (SovereignKey, DvdMintKey, DoveMintKey);

    #[cfg(feature = "wasm")]
    fn get_accounts(program_key: ProgramKey, keys: Self::Keys) -> Vec<AccountMeta> {
        let (sovereign_key, dvd_mint_key, dove_mint_key) = keys;
        vec![
            AccountMeta {
                pubkey: *sovereign_key,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: program_key.derive_world(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: program_key.derive_authority(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: *dvd_mint_key,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: *dove_mint_key,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: solana_program::system_program::ID,
                is_signer: false,
                is_writable: false,
            },
        ]
    }

    fn execute(self, program_id: &'static Pubkey, accounts: &'static [AccountInfo]) {
        let sovereign_account = Signer::new(&accounts[0]);
        let world_account = Writable::new(&accounts[1]);
        let authority_account = Readonly::new(&accounts[2]);
        let dvd_mint_account = MintAccount::new(Readonly::new(&accounts[3]));
        let dove_mint_account = MintAccount::new(Readonly::new(&accounts[4]));
        let system_program_account = SystemProgramAccount::new(&accounts[5]);

        let authority = Authority::from_account(program_id, authority_account);

        let clock = Clock::get().map_err(|_| "Failed to get clock").unwrap();
        let rent = Rent::get().map_err(|_| "Failed to get rent").unwrap();
        World::create(
            program_id,
            sovereign_account,
            world_account,
            system_program_account,
            (),
            &rent,
            WorldParams {
                sovereign_account,
                dove_mint_account,
                dvd_mint_account,
                authority,
                clock,
                vesting_recipient: self.vesting_recipient,
                vesting_schedule: self.vesting_schedule,
            },
        );
    }
}
