use {
    std::{
        fmt::Display,
        ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    },
    uint::construct_uint,
};

/// Identity (10^18)
const WAD: u128 = 1_000_000_000_000_000_000;

construct_uint! {
    pub struct U192(3);
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Positive decimal values, precise to 18 digits, with 68 bits of integer precision
/// Maximum value is (2^128 - 1) / 10^18 = 3.4028 * 10^20
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Decimal(u128);

impl Decimal {
    pub const fn one() -> Self {
        Self(WAD)
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Saturating subtraction
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    /// Calculates base^exp
    pub fn pow(self, mut exp: u64) -> Self {
        let mut base = self;
        let mut ret = if exp % 2 != 0 { base } else { Self(WAD) };

        while exp > 0 {
            exp /= 2;
            base = base * base;

            if exp % 2 != 0 {
                ret = ret * base;
            }
        }

        ret
    }
    pub fn from_token_amount(amount: u64, decimals: u8) -> Self {
        if decimals > 18 {
            panic!("Decimals must be 18 or less");
        }
        Self::from(amount) / Self::from(10u64.pow(decimals as u32))
    }

    /// Conversion to token amount
    pub fn to_token_amount(self, decimals: u8) -> u64 {
        if decimals > 18 {
            panic!("Decimals must be 18 or less");
        }
        let scale_factor = 10u128.pow(decimals as u32);
        if scale_factor > WAD {
            panic!("Too many decimals");
        }
        (self.0 / (WAD / scale_factor))
            .try_into()
            .expect("Overflow in to_token_amount conversion to u64")
    }

    /// Takes the value from this `Decimal` and replaces it with zero.
    pub fn take(&mut self) -> Decimal {
        let val = *self;
        *self = Decimal::zero();
        val
    }

    #[cfg(test)]
    /// Returns the absolute difference between two `Decimal` values
    pub fn abs_diff(self, rhs: Self) -> Self {
        Self(self.0.abs_diff(rhs.0))
    }
}

#[cfg(feature = "wasm")]
#[allow(non_snake_case)]
#[wasm_bindgen]
impl Decimal {
    #[wasm_bindgen(js_name = "tokenAmountToNumber")]
    pub fn token_amount_to_number(amount: u64, decimals: u8) -> f64 {
        Self::from_token_amount(amount, decimals).to_f64()
    }

    #[wasm_bindgen(js_name = "numberToTokenAmount")]
    pub fn number_to_token_amount(amount: f64, decimals: u8) -> u64 {
        Self::from(amount).to_token_amount(decimals)
    }
}

impl From<u64> for Decimal {
    fn from(val: u64) -> Self {
        Self(val as u128 * WAD)
    }
}

impl From<f64> for Decimal {
    fn from(mut val: f64) -> Self {
        if val.is_nan() || val.is_infinite() || val < 0.0 {
            val = 0.0;
        }
        Self((val * (WAD as f64)) as u128)
    }
}

#[cfg(feature = "wasm")]
impl Decimal {
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / WAD as f64
    }
}

impl Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{:018}", self.0 / WAD, self.0 % WAD)
    }
}

impl Add for Decimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0.checked_add(rhs.0).expect("Overflow in add"))
    }
}

impl Add<u64> for Decimal {
    type Output = Self;

    fn add(self, rhs: u64) -> Self {
        self + Self::from(rhs)
    }
}

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0.checked_sub(rhs.0).expect("Overflow in sub"))
    }
}

impl Sub<u64> for Decimal {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self {
        self - Self::from(rhs)
    }
}

impl Mul for Decimal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let result = (U192::from(self.0) * U192::from(rhs.0)) / U192::from(WAD);
        Self(result.try_into().expect("Overflow in mul"))
    }
}

impl Mul<u64> for Decimal {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self {
        Self(self.0.checked_mul(rhs as u128).expect("Overflow in mul"))
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        let result = (U192::from(self.0) * U192::from(WAD)) / U192::from(rhs.0);
        Self(u128::try_from(result).expect("Overflow in div"))
    }
}

impl Div<u64> for Decimal {
    type Output = Self;

    fn div(self, rhs: u64) -> Self {
        Self(self.0.checked_div(rhs as u128).expect("Overflow in div"))
    }
}

impl AddAssign for Decimal {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign<u64> for Decimal {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl SubAssign for Decimal {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign<u64> for Decimal {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl MulAssign for Decimal {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl MulAssign<u64> for Decimal {
    fn mul_assign(&mut self, rhs: u64) {
        *self = *self * rhs;
    }
}

impl DivAssign for Decimal {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl DivAssign<u64> for Decimal {
    fn div_assign(&mut self, rhs: u64) {
        *self = *self / rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_from() {
        assert_eq!(Decimal::from(1), Decimal(WAD));
        assert_eq!(Decimal::from(2), Decimal(2 * WAD));
    }

    #[test]
    fn test_decimal_add() {
        let a = Decimal::from(5);
        let b = Decimal::from(7);
        assert_eq!(a + b, Decimal::from(12));
        assert_eq!(a + 7u64, Decimal::from(12));
    }

    #[test]
    fn test_decimal_sub() {
        let a = Decimal::from(10);
        let b = Decimal::from(3);
        assert_eq!(a - b, Decimal::from(7));
        assert_eq!(a - 3u64, Decimal::from(7));
    }

    #[test]
    fn test_decimal_mul() {
        let a = Decimal::from(5);
        let b = Decimal::from(3);
        assert_eq!(a * b, Decimal::from(15));
        assert_eq!(a * 3u64, Decimal::from(15));
    }

    #[test]
    fn test_decimal_div() {
        let a = Decimal::from(15);
        let b = Decimal::from(3);
        assert_eq!(a / b, Decimal::from(5));
        assert_eq!(a / 3u64, Decimal::from(5));
    }

    #[test]
    fn test_decimal_assign_ops() {
        let mut a = Decimal::from(5);

        a += Decimal::from(3);
        assert_eq!(a, Decimal::from(8));

        a -= Decimal::from(2);
        assert_eq!(a, Decimal::from(6));

        a *= Decimal::from(2);
        assert_eq!(a, Decimal::from(12));

        a /= Decimal::from(3);
        assert_eq!(a, Decimal::from(4));

        a += 1u64;
        assert_eq!(a, Decimal::from(5));

        a -= 1u64;
        assert_eq!(a, Decimal::from(4));

        a *= 2u64;
        assert_eq!(a, Decimal::from(8));

        a /= 2u64;
        assert_eq!(a, Decimal::from(4));
    }
}
