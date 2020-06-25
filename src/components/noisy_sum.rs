use crate::{Domain, PrivacyLoss, Measurement};
use crate::traits::{GenRand, PartialMax};
use std::iter::Sum;
use num::traits::Signed;
use std::fmt::Debug;
use num::Zero;

/// Create a measurement struct representing a noisy sum.
/// In Whitenoise these bounds do not have to be passed multiple times
pub fn make_noisy_sum<T>(
    lower: T, upper: T, epsilon: PrivacyLoss
) -> Result<Measurement<T, T>, &'static str>
    where T: 'static + GenRand + Signed + Clone + PartialOrd + Sum + PartialMax + Debug + Zero + From<f64> {

    if lower > upper {
        return Err("lower must not be greater than upper")
    }
    if epsilon <= 0. {
        return Err("epsilon must be positive")
    }
    Ok(Measurement {
        input_domain: Domain::<T>::Continuous { lower: Some(lower.clone()), upper: Some(upper.clone()) },
        privacy_loss: epsilon,
        function: Box::new(move |data: Vec<T>| {
            let aggregated = data.into_iter().sum::<T>();

            let sensitivity = lower.abs().partial_max(&upper.abs())?;
            let noise = T::sample_laplace(T::zero(), sensitivity / epsilon.into())?.into();

            Ok(aggregated + noise)
        }),
    })
}


#[cfg(test)]
mod tests {
    use crate::components::noisy_sum::make_noisy_sum;

    #[test]
    fn test_noisy_sum() {
        let noisy_sum_measure = make_noisy_sum(0.5, 1.5, 0.4).unwrap();

        let num_records = 100;
        let data = (0..num_records).map(|_| 1.).collect::<Vec<f64>>();

        let release = (noisy_sum_measure.function)(data).unwrap();

        println!("{:?}", release);
    }
}
