//! Various implementations of Measurement.
//!
//! The different [`Measurement`] implementations in this module are accessed by calling the appropriate constructor function.
//! Constructors are named in the form `make_xxx()`, where `xxx` indicates what the resulting `Measurement` does.

use rand::Rng;

use crate::core::Measurement;
use crate::dist::{L1Sensitivity, MaxDivergence};
use crate::dom::AllDomain;

fn laplace(sigma: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(-0.5, 0.5);
    u.signum() * (1.0 - 2.0 * u.abs()).ln() * sigma
}

pub trait AddNoise {
    fn add_noise(self, noise: f64) -> Self;
}
impl AddNoise for u32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for u64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for i32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for i64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for f32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for f64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for u8 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }

pub fn make_base_laplace<T>(sigma: f64) -> Measurement<AllDomain<T>, AllDomain<T>, L1Sensitivity<f64>, MaxDivergence> where
    T: Copy + AddNoise {
    let input_domain = AllDomain::new();
    let output_domain = AllDomain::new();
    let function = move |arg: &T| -> T {
        let noise = laplace(sigma);
        arg.add_noise(noise)
    };
    let input_metric = L1Sensitivity::new();
    let output_measure = MaxDivergence::new();
    let privacy_relation = move |d_in: &f64, d_out: &f64| *d_out >= *d_in / sigma;
    Measurement::new(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_base_laplace() {
        let measurement = make_base_laplace::<f64>(1.0);
        let arg = 0.0;
        let _ret = measurement.function.eval(&arg);
        // TODO: Test for base_laplace
    }

}
