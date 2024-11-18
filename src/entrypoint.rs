use {
    crate::{
        commands::{
            AuthorityCreate, CollateralCreate, CollateralSetOracle, CollateralUpdateMaxDeposit,
            ConfigUpdate, FlashMintBegin, FlashMintEnd, OfferingBuy, OfferingEnd, OfferingStart,
            SavingsClaimRewards, SavingsCreate, SavingsDeposit, SavingsWithdraw, SovereignUpdate,
            StabilityBuyDvd, StabilityCreate, StabilitySellDvd, StabilityUpdateMaxDeposit,
            UserFeedCreate, UserFeedSetPrice, VaultBorrow, VaultBuyCollateral, VaultClaimRewards,
            VaultCreate, VaultCreateReserve, VaultDeposit, VaultFailAuction, VaultLiquidate,
            VaultRemoveReserve, VaultRepay, VaultUnliquidate, VaultWithdraw, VestingClaim,
            VestingUpdateRecipient, WorldCreate,
        },
        traits::{Command, Pod},
        util::revert,
    },
    solana_program::{
        account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
    },
};

#[cfg_attr(rustfmt, rustfmt_skip)]
pub fn process_instruction(
    program_id: &'static Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    // Safety: This transmute is safe because the `accounts` slice
    // is guaranteed to live for the duration of `process_instruction`.
    // It allows us to treat the slice as 'static for convenience.
    let accounts: &'static [AccountInfo] = unsafe { core::mem::transmute(accounts) };

    if instruction_data.len() < 4 {
        revert("instruction data too short");
    }
    let (id_data, cmd_data) = instruction_data.split_at(4);
    let id = u32::from_le_bytes(id_data.try_into().unwrap());
    match id {
        AuthorityCreate::ID => AuthorityCreate::cast_from(cmd_data).execute(program_id, accounts),
        CollateralCreate::ID => CollateralCreate::cast_from(cmd_data).execute(program_id, accounts),
        CollateralSetOracle::ID => CollateralSetOracle::cast_from(cmd_data).execute(program_id, accounts),
        CollateralUpdateMaxDeposit::ID => CollateralUpdateMaxDeposit::cast_from(cmd_data).execute(program_id, accounts),
        ConfigUpdate::ID => ConfigUpdate::cast_from(cmd_data).execute(program_id, accounts),
        FlashMintBegin::ID => FlashMintBegin::cast_from(cmd_data).execute(program_id, accounts),
        FlashMintEnd::ID => FlashMintEnd::cast_from(cmd_data).execute(program_id, accounts),
        OfferingBuy::ID => OfferingBuy::cast_from(cmd_data).execute(program_id, accounts),
        OfferingEnd::ID => OfferingEnd::cast_from(cmd_data).execute(program_id, accounts),
        OfferingStart::ID => OfferingStart::cast_from(cmd_data).execute(program_id, accounts),
        SavingsClaimRewards::ID => SavingsClaimRewards::cast_from(cmd_data).execute(program_id, accounts),
        SavingsCreate::ID => SavingsCreate::cast_from(cmd_data).execute(program_id, accounts),
        SavingsDeposit::ID => SavingsDeposit::cast_from(cmd_data).execute(program_id, accounts),
        SavingsWithdraw::ID => SavingsWithdraw::cast_from(cmd_data).execute(program_id, accounts),
        SovereignUpdate::ID => SovereignUpdate::cast_from(cmd_data).execute(program_id, accounts),
        StabilityCreate::ID => StabilityCreate::cast_from(cmd_data).execute(program_id, accounts),
        StabilityBuyDvd::ID => StabilityBuyDvd::cast_from(cmd_data).execute(program_id, accounts),
        StabilitySellDvd::ID => StabilitySellDvd::cast_from(cmd_data).execute(program_id, accounts),
        StabilityUpdateMaxDeposit::ID => StabilityUpdateMaxDeposit::cast_from(cmd_data).execute(program_id, accounts),
        UserFeedCreate::ID => UserFeedCreate::cast_from(cmd_data).execute(program_id, accounts),
        UserFeedSetPrice::ID => UserFeedSetPrice::cast_from(cmd_data).execute(program_id, accounts),
        VaultBorrow::ID => VaultBorrow::cast_from(cmd_data).execute(program_id, accounts),
        VaultBuyCollateral::ID => VaultBuyCollateral::cast_from(cmd_data).execute(program_id, accounts),
        VaultClaimRewards::ID => VaultClaimRewards::cast_from(cmd_data).execute(program_id, accounts),
        VaultCreate::ID => VaultCreate::cast_from(cmd_data).execute(program_id, accounts),
        VaultCreateReserve::ID => VaultCreateReserve::cast_from(cmd_data).execute(program_id, accounts),
        VaultDeposit::ID => VaultDeposit::cast_from(cmd_data).execute(program_id, accounts),
        VaultFailAuction::ID => VaultFailAuction::cast_from(cmd_data).execute(program_id, accounts),
        VaultLiquidate::ID => VaultLiquidate::cast_from(cmd_data).execute(program_id, accounts),
        VaultRemoveReserve::ID => VaultRemoveReserve::cast_from(cmd_data).execute(program_id, accounts),
        VaultRepay::ID => VaultRepay::cast_from(cmd_data).execute(program_id, accounts),
        VaultUnliquidate::ID => VaultUnliquidate::cast_from(cmd_data).execute(program_id, accounts),
        VaultWithdraw::ID => VaultWithdraw::cast_from(cmd_data).execute(program_id, accounts),
        VestingClaim::ID => VestingClaim::cast_from(cmd_data).execute(program_id, accounts),
        VestingUpdateRecipient::ID => VestingUpdateRecipient::cast_from(cmd_data).execute(program_id, accounts),
        WorldCreate::ID => WorldCreate::cast_from(cmd_data).execute(program_id, accounts),
        _ => revert("Invalid command ID"),
    };
    Ok(())
}

entrypoint!(process_instruction);
