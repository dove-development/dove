/// The Pyth price oracle.
/// This object will report the slot-weighted, inverse confidence-weighted
/// exponential moving average price of the specified asset over the last 5921 secs.
use {
    crate::{
        finance::Decimal,
        util::Time,
    },
    pyth_sdk_solana::state::load_price_account,
    solana_program::pubkey::Pubkey,
};

pub struct Pyth;
impl Pyth {
    pub fn query(key: &Pubkey, data: &[u8]) -> Result<(Decimal, Time), &'static str> {
        let price_feed = load_price_account::<32, ()>(data)
            .map_err(|_| "could not load pyth price feed")?
            .to_price_feed(key);

        let ema = price_feed.get_ema_price_unchecked();
        if ema.price < 0 {
            return Err("pyth price is negative");
        }
        let price = ema.price as u64;
        // at least 90% confidence
        if ema.conf.saturating_mul(10) > price {
            return Err("pyth price has too low confidence");
        }

        // take lower bound
        let base = Decimal::from(price - ema.conf);
        let scale_factor = Decimal::from(10).pow(ema.expo.abs() as u64);
        let price = if ema.expo >= 0 {
            base * scale_factor
        } else {
            base / scale_factor
        };
        let time = Time::from_unix_timestamp(ema.publish_time as u64);
        Ok((price, time))
    }
}
