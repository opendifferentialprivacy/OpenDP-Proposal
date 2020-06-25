use crate::{Domain, PrivacyLoss, Measurement, Properties};
use crate::traits::{PartialMax, GenLaplace};
use std::iter::Sum;
use num::traits::Signed;
use std::fmt::Debug;
use num::Zero;

/// Create a measurement struct representing a noisy sum.
/// In Whitenoise these bounds do not have to be passed multiple times
pub fn make_noisy_sum<T>(
    input_properties: Properties<T>, epsilon: PrivacyLoss,
) -> Result<Measurement<T, T>, &'static str>
    where T: 'static + GenLaplace + Signed + Clone + PartialOrd + Sum + PartialMax + Debug + Zero + From<f64> {

    let (lower, upper) = match &input_properties.domain {
        Some(Domain::Continuous { lower, upper }) => match (lower, upper) {
            (Some(lower), Some(upper)) => if lower > upper {
                return Err("lower must not be greater than upper");
            } else { (lower.clone(), upper.clone()) },
            _ => return Err("lower and upper must be defined")
        },
        _ => return Err("domain must be continuous")
    };

    if input_properties.has_nullity {
        return Err("input data must not contain nullity")
    }

    if epsilon <= 0. {
        return Err("epsilon must be positive");
    }

    Ok(Measurement {
        input_properties,
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
    use crate::{Properties, Domain};

    #[test]
    fn test_noisy_sum() {

        let input_properties = Properties {
            has_nullity: false,
            domain: Some(Domain::Continuous {
                lower: Some(0.5),
                upper: Some(1.5)
            })
        };

        let noisy_sum_measure = make_noisy_sum(input_properties, 0.4).unwrap();

        let num_records = 100;
        let data = (0..num_records).map(|_| 1.).collect::<Vec<f64>>();

        let release = (noisy_sum_measure.function)(data).unwrap();

        println!("{:?}", release);
    }
}
