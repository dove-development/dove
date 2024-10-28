/// The Pyth price oracle.
/// This object will report the slot-weighted, inverse confidence-weighted
/// exponential moving average price of the specified asset over the last 5921 secs.
use {
    crate::{finance::Decimal, traits::Pod, util::Time},
    pyth_solana_receiver_sdk::price_update::{PriceFeedMessage, PriceUpdateV2},
    solana_program::pubkey::Pubkey,
    switchboard_solana::AnchorDeserialize,
};

pub struct Pyth;
impl Pyth {
    pub fn query(data: &[u8], owner: &Pubkey) -> Result<(Decimal, Time), &'static str> {
        if owner.as_bytes() != pyth_solana_receiver_sdk::ID_CONST.to_bytes() {
            return Err("pyth oracle not owned by pyth");
        }

        let price_feed = PriceUpdateV2::deserialize(&mut &data[8..])
            .map_err(|_| "could not load pyth price feed")?;

        let PriceFeedMessage {
            exponent,
            publish_time,
            ema_price,
            ema_conf,
            ..
        } = price_feed.price_message;

        if ema_price < 0 {
            return Err("pyth price is negative");
        }
        let price = ema_price as u64;
        // at least 90% confidence
        if ema_conf.saturating_mul(10) > price {
            return Err("pyth price has too low confidence");
        }

        // take lower bound
        let base = Decimal::from(price - ema_conf);
        let scale_factor = Decimal::from(10).pow(exponent.abs() as u64);
        let price = if exponent >= 0 {
            base * scale_factor
        } else {
            base / scale_factor
        };

        if publish_time < 0 {
            return Err("pyth publish time is negative");
        }
        let time = Time::from_unix_timestamp(publish_time as u64);
        Ok((price, time))
    }
}
