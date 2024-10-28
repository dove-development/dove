/// The Switchboard price oracle.
/// This object will report the live price of the specified asset.
use {
    crate::{finance::Decimal, util::Time},
    solana_program::pubkey::Pubkey,
};
// `switchboard_solana` is not supported in WebAssembly builds
// due to linkage conflicts between solana-program v1 and v2.
// So, we just use a dummy implementation there (oracles aren't available anyway).
#[cfg(not(feature = "wasm"))]
use {crate::traits::Pod, switchboard_solana::AggregatorAccountData};

pub struct Switchboard;
impl Switchboard {
    #[cfg(feature = "wasm")]
    pub fn query(_: &[u8], _: &Pubkey) -> Result<(Decimal, Time), &'static str> {
        Err("switchboard oracle not supported in wasm, use sw js lib")
    }

    #[cfg(not(feature = "wasm"))]
    pub fn query(data: &[u8], owner: &Pubkey) -> Result<(Decimal, Time), &'static str> {
        if owner.as_bytes() != switchboard_solana::ID_CONST.to_bytes() {
            return Err("switchboard oracle not owned by switchboard");
        }
        let feed = AggregatorAccountData::new_from_bytes(data)
            .map_err(|_| "could not load switchboard aggregator account data")?;
        let price = feed
            .get_result()
            .map_err(|_| "could not get switchboard result")?;
        if price.mantissa < 0 {
            return Err("switchboard oracle price is negative which is not allowed");
        }
        let base = Decimal::from(price.mantissa as u64);
        let exp = Decimal::from(10u64.pow(price.scale));
        let price = base / exp;
        let time = Time::from_unix_timestamp(feed.current_round.round_open_timestamp as u64);
        Ok((price, time))
    }
}
