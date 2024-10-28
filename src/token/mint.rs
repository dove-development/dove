use {
    crate::{
        accounts::{MintAccount, Readonly, Signer, TokenAccount, TokenProgramAccount, Writable},
        traits::Account,
        util::{require, Expect},
    },
    solana_program::{
        program::{invoke, invoke_signed},
        program_option::COption,
        program_pack::Pack,
        pubkey::Pubkey,
    },
    spl_token::state::Mint as SplMint,
};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Mint {
    key: Pubkey,
}

impl Mint {
    pub fn from_account(
        mint_account: MintAccount<Readonly>,
        expected_mint_authority: Expect<Readonly>,
        expected_freeze_authority: Expect<Readonly>,
        expected_supply: Expect<u64>,
        decimals_out: &mut u8,
        supply_out: &mut u64
    ) -> Self {
        let mint_data = mint_account.get_info().data.borrow();
        let mint = SplMint::unpack(&mint_data)
            .map_err(|_| "couldn't unpack mint data")
            .unwrap();
        match expected_mint_authority {
            Expect::Some(mint_authority) => {
                let actual = mint.mint_authority;
                let expected = COption::Some(*mint_authority.get_info().key);
                require(
                    actual == expected,
                    "Mint has a mint authority that does not match the expected mint authority",
                );
            }
            Expect::None => {
                require(
                    mint.mint_authority.is_none(),
                    "Mint has a mint authority when none was expected",
                );
            }
            Expect::Any => {}
        }
        match expected_freeze_authority {
            Expect::Some(freeze_authority) => {
                let actual = mint.freeze_authority;
                let expected = COption::Some(*freeze_authority.get_info().key);
                require(
                    actual == expected,
                    "Mint has a freeze authority that does not match the expected freeze authority",
                );
            }
            Expect::None => {
                require(
                    mint.freeze_authority.is_none(),
                    "Mint has a freeze authority when none was expected",
                );
            }
            Expect::Any => {}
        }
        match expected_supply {
            Expect::Some(expected) => {
                require(
                    mint.supply == expected,
                    "Mint has a supply that does not match the expected supply",
                );
            }
            Expect::None => {
                require(
                    mint.supply == 0,
                    "Mint has a non-zero supply when zero was expected",
                );
            }
            Expect::Any => {}
        }
        *decimals_out = mint.decimals;
        *supply_out = mint.supply;
        Self {
            key: *mint_account.get_info().key,
        }
    }

    pub fn get_key(&self) -> &Pubkey {
        &self.key
    }

    pub fn mint(
        &self,
        mint_account: MintAccount<Writable>,
        token_account: TokenAccount<Writable>,
        mint_authority: Readonly,
        token_program_account: TokenProgramAccount,
        amount: u64,
        seeds: &[&[u8]],
    ) {
        let mint_info = mint_account.get_info();
        let token_account_info = token_account.get_info();
        let mint_authority_info = mint_authority.get_info();
        require(mint_info.key == &self.key, "mint_info key mismatch");
        _ = token_program_account; // must be included when calling into spl_token
        invoke_signed(
            &spl_token::instruction::mint_to(
                &spl_token::ID,
                &mint_info.key,
                token_account_info.key,
                mint_authority_info.key,
                &[],
                amount,
            )
            .map_err(|_| "couldn't create mint_to instruction")
            .unwrap(),
            &[
                mint_info.clone(),
                token_account_info.clone(),
                mint_authority_info.clone(),
            ],
            &[seeds],
        )
        .map_err(|_| "couldn't mint to target")
        .unwrap()
    }

    pub fn burn(
        &self,
        mint_account: MintAccount<Writable>,
        token_account: TokenAccount<Writable>,
        token_program_account: TokenProgramAccount,
        user_account: Signer,
        amount: u64,
    ) {
        let mint_info = mint_account.get_info();
        let token_account_info = token_account.get_info();
        let user_info = user_account.get_info();
        require(mint_info.key == &self.key, "mint_info key mismatch");
        _ = token_program_account; // must be included when calling into spl_token
        invoke(
            &spl_token::instruction::burn(
                &spl_token::ID,
                token_account_info.key,
                &mint_info.key,
                user_info.key,
                &[],
                amount,
            )
            .map_err(|_| "couldn't create burn instruction")
            .unwrap(),
            &[
                token_account_info.clone(),
                mint_info.clone(),
                user_info.clone(),
            ],
        )
        .map_err(|_| "couldn't burn tokens")
        .unwrap()
    }
}
