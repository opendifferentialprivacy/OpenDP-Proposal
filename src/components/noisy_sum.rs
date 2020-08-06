use crate::{Bounds, Measurement, Domain, Distance};
use crate::traits::{PartialMax, GenLaplace};
use std::iter::Sum;
use num::traits::Signed;
use std::fmt::Debug;
use num::Zero;



/// Calculates "NoisyStat", which adds Laplace noise to the OLS sufficient statistics
///
pub fn noisy_stats<I>(xys: I, x_mean: Float, y_mean: Float, epsilon: Float) -> Result<(Float, Float), Error>
    where
        I: Iterator<Item=(Float, Float)>,
{

    let data_size_hint: (usize, Option<usize>) = xys.size_hint();

    let data_size: Float = data_size_hint.0 as Float;

    let delta: Float = 1.0 - 1.0 / data_size;

    let laplace_1: Float = laplace_mechanism(epsilon, 3.0*delta, false).unwrap();
    let laplace_2: Float = laplace_mechanism(epsilon, 3.0*delta, false).unwrap();

    // SUM (x-mean(x))^2
    let mut xxm2 = 0.0;

    // SUM (x-mean(x)) (y-mean(y))
    let mut xmym2 = 0.0;

    for (x, y) in xys {
        xxm2 = xxm2 + (x - x_mean) * (x - x_mean);
        xmym2 = xmym2 + (x - x_mean) * (y - y_mean);
    }

    let slope = (xmym2 + laplace_1) / (xxm2 + laplace_2);

    let delta_2 = (1.0 / data_size) * (1.0 + slope.abs());

    let laplace_3: Float = laplace_mechanism(epsilon, 3.0*delta_2, false).unwrap();

    let intercept = y_mean - slope * x_mean + laplace_3;

    let p_25 = 0.25 * slope + intercept;
    let p_75 = 0.75 * slope + intercept;

    // we check for divide-by-zero after the fact
    if slope.is_nan() {
        return Err(Error::TooSteep);
    }

    Ok((p_25, p_75))
}


/// Create a measurement struct representing a noisy sum.
/// In Whitenoise these bounds do not have to be passed multiple times
pub fn make_noisy_sum<T>(
    input_properties: Domain<T>, epsilon: PrivacyLoss,
) -> Result<Measurement<T, T>, &'static str>
    where T: 'static + GenLaplace + Signed + Clone + PartialOrd + Sum + PartialMax + Debug + Zero + From<f64> {

    let (lower, upper) = match &input_properties.bounds {
        Some(Bounds::Continuous { lower, upper }) => match (lower, upper) {
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
        input_domain: input_properties,
        privacy_relation: Box::new(move |dist_in, dist_out| {
            match input_properties {
                Some(Bounds::Continuous {lower: x, upper: y}) =>
            }
            match (dist_in, dist_out) {
                (Distance::Symmetric(dist_in), Distance::L2(dist_out)) => {
                    input_properties.
                }
            }
        }),
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
    use crate::{Domain, Bounds};

    #[test]
    fn test_noisy_sum() {

        let input_properties = Domain {
            has_nullity: false,
            bounds: Some(Bounds::Continuous {
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
