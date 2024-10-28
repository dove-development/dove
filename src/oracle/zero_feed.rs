use crate::{finance::Decimal, util::Time};

pub struct ZeroFeed;

impl ZeroFeed {
    pub const fn query(time: Time) -> Result<(Decimal, Time), &'static str> {
        Ok((Decimal::zero(), time))
    }
}
