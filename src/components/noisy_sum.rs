use crate::{Domain, Scalar, MultiSet, Float, PrivacyLoss, Measurement};
use crate::utilities::sample_laplace;

/// Create a measurement struct representing a noisy sum.
/// In Whitenoise these bounds do not have to be passed multiple times
pub fn make_noisy_sum(lower: Scalar, upper: Scalar, epsilon: PrivacyLoss) -> Result<Measurement, &'static str> {
    if lower > upper {
        return Err("lower must not be greater than upper")
    }
    if epsilon <= 0. {
        return Err("epsilon must be positive")
    }
    Ok(Measurement {
        input_domain: Domain::Continuous { lower: Some(lower.clone()), upper: Some(upper.clone()) },
        privacy_loss: epsilon,
        function: Box::new(move |data: &MultiSet| {
            match (lower.clone(), upper.clone(), data) {
                (Scalar::Float(lower), Scalar::Float(upper), MultiSet::Float(data)) => {
                    let aggregated = data.into_iter().sum::<Float>();

                    let sensitivity = lower.abs().max(upper.abs());
                    let noise: Float = sample_laplace(sensitivity / epsilon).into();

                    Ok(Scalar::Float(aggregated + noise))
                }

                (Scalar::Integer(_), Scalar::Integer(_), MultiSet::Integer(_)) =>
                    unimplemented!(),

                _ => Err("noisy_sum: data needs to be numeric and data/bounds homogeneously typed")
            }
        }),
    })
}


#[cfg(test)]
mod tests {
    use crate::components::noisy_sum::make_noisy_sum;

    #[test]
    fn test_noisy_sum() {
        let noisy_sum_measure = make_noisy_sum(0.5.into(), 1.5.into(), 0.4).unwrap();

        let num_records = 100;
        let data = (0..num_records).map(|_| 1.).collect::<Vec<f64>>();

        let release = (noisy_sum_measure.function)(&data.into()).unwrap();

        println!("{:?}", release);
    }
}
