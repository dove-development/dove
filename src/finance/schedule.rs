use crate::finance::Decimal;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Clone, Copy, Default)]
#[repr(C)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Schedule {
    maximum: Decimal,
    warmup_length: Decimal,
    total_length: Decimal,
}

impl Schedule {
    pub fn integrate(&self, t1: Decimal, t2: Decimal) -> Decimal {
        let w = self.warmup_length;
        let l = self.total_length;
        let m = self.maximum;

        if t1 >= t2 || t1 >= l {
            return Decimal::zero();
        }

        // clamp t2 to the total length
        let t2 = if t2 > l { l } else { t2 };

        if t1 >= w {
            // rectangle, both t1 and t2 are in the post-warmup period
            m * (t2 - t1)
        } else if t2 <= w {
            // trapezoid, t1 and t2 are both in the warmup period
            let left = m * (t1 / w);
            let right = m * (t2 / w);
            let bottom = t2 - t1;
            ((left + right) / 2) * bottom
        } else {
            // trapezoid + rectangle, t2 is in the post-warmup period and t1 is in the warmup period
            self.integrate(t1, w) + self.integrate(w, t2)
        }
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Schedule {
    #[wasm_bindgen(constructor)]
    #[allow(non_snake_case)]
    pub fn new(maximum: f64, warmupLength: f64, totalLength: f64) -> Result<Self, String> {
        if maximum <= 0.0 {
            return Err("Maximum must be positive".to_string());
        }
        if warmupLength <= 0.0 {
            return Err("Warmup length must be positive".to_string());
        }
        if totalLength <= 0.0 {
            return Err("Total length must be positive".to_string());
        }
        if warmupLength > totalLength {
            return Err("Warmup length cannot be greater than total length".to_string());
        }
        Ok(Self {
            maximum: Decimal::from(maximum),
            warmup_length: Decimal::from(warmupLength),
            total_length: Decimal::from(totalLength),
        })
    }

    #[wasm_bindgen(getter)]
    pub fn maximum(&self) -> f64 {
        self.maximum.to_f64()
    }

    #[wasm_bindgen(getter, js_name = warmupLength)]
    pub fn warmup_length(&self) -> f64 {
        self.warmup_length.to_f64()
    }

    #[wasm_bindgen(getter, js_name = totalLength)]
    pub fn total_length(&self) -> f64 {
        self.total_length.to_f64()
    }

    pub fn at(&self, t: f64) -> f64 {
        let t = Decimal::from(t);
        let w = self.warmup_length;
        let l = self.total_length;
        let m = self.maximum;

        if t >= l {
            return 0.0;
        } else if t >= w {
            return m.to_f64();
        } else {
            return (m * (t / w)).to_f64();
        }
    }

    #[wasm_bindgen(getter, js_name = totalEmission)]
    pub fn total_emission(&self) -> f64 {
        self.integrate(Decimal::zero(), self.total_length).to_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrate() {
        let schedule = Schedule {
            maximum: Decimal::from(10),
            warmup_length: Decimal::from(5),
            total_length: Decimal::from(15),
        };

        assert_eq!(
            schedule.integrate(Decimal::from(0), Decimal::from(0)),
            Decimal::from(0)
        );
        assert_eq!(
            schedule.integrate(Decimal::from(0), Decimal::from(2.5)),
            Decimal::from(6.25)
        );
        assert_eq!(
            schedule.integrate(Decimal::from(0), Decimal::from(5)),
            Decimal::from(25)
        );
        assert_eq!(
            schedule.integrate(Decimal::from(0), Decimal::from(10)),
            Decimal::from(75)
        );
        assert_eq!(
            schedule.integrate(Decimal::from(0), Decimal::from(15)),
            Decimal::from(125)
        );
        assert_eq!(
            schedule.integrate(Decimal::from(0), Decimal::from(20)),
            Decimal::from(125)
        );
        assert_eq!(
            schedule.integrate(Decimal::from(5), Decimal::from(10)),
            Decimal::from(50)
        );
        assert_eq!(
            schedule.integrate(Decimal::from(5), Decimal::from(15)),
            Decimal::from(100)
        );
        assert_eq!(
            schedule.integrate(Decimal::from(10), Decimal::from(15)),
            Decimal::from(50)
        );
        assert_eq!(
            schedule.integrate(Decimal::from(15), Decimal::from(20)),
            Decimal::from(0)
        );
        assert_eq!(
            schedule.integrate(Decimal::from(20), Decimal::from(25)),
            Decimal::from(0)
        );
    }

    #[test]
    fn test_integrate_consistency() {
        let schedule = Schedule {
            maximum: Decimal::from(10),
            warmup_length: Decimal::from(5),
            total_length: Decimal::from(15),
        };

        let test_points = vec![
            Decimal::from(0),
            Decimal::from(2.5),
            Decimal::from(5),
            Decimal::from(7.5),
            Decimal::from(10),
            Decimal::from(12.5),
            Decimal::from(15),
            Decimal::from(20),
        ];
        let epsilon = Decimal::from(0.000001);

        for &x in &test_points {
            for &z in &test_points {
                let left = schedule.integrate(Decimal::from(0), x + z);
                let right = schedule.integrate(Decimal::from(0), x) + schedule.integrate(x, x + z);
                assert!(
                    left.abs_diff(right) < epsilon,
                    "Inconsistency found: |integrate(0, {}) - (integrate(0, {}) + integrate({}, {}))| >= {}",
                    x + z,
                    x,
                    x,
                    x + z,
                    epsilon
                );
            }
        }
    }
}
