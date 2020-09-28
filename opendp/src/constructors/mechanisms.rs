use crate::base::domain::Domain;
use crate::base::value::{Scalar, Value};
use crate::{Transformation, Error, Measurement};
use crate::base::metric::{PrivacyDistance, DataDistance};
use num::{NumCast, ToPrimitive};
use num::cast::cast;
use crate::base::Data;
use crate::base::value::*;
use noisy_float::types::R64;

fn to_f64<T: NumCast + Clone>(v: T) -> Result<f64, Error> {
    cast::<T, R64>(v).ok_or_else(|| Error::UnsupportedCast)?
        .to_f64().ok_or_else(|| Error::UnsupportedCast)
}

fn relation_gaussian_mechanism<T: PartialOrd + NumCast>(
    sensitivity: T, sigma: T, epsilon: f64, delta: f64
) -> Result<bool, Error> {
    let sensitivity: f64 = cast::<T, f64>(sensitivity).ok_or_else(|| Error::UnsupportedCast)?;
    let sigma: f64 = cast::<T, f64>(sigma).ok_or_else(|| Error::UnsupportedCast)?;

    if delta < 0. {
        return Err(Error::Raw("delta may not be less than zero".to_string()))
    }
    Ok(epsilon.min(1.) >= (sensitivity / sigma) * ((1.25 / delta).ln() * 2.).sqrt())
}

pub fn gaussian_mechanism(
    value: f64,
    sigma: f64
) -> Result<f64, Error> {
    if sigma <= 0. {
        return Err(Error::Raw(format!("sigma ({}) be positive", sigma)));
        // return Err(Error::Raw(format!("sigma ({}) be positive", sigma).as_str()));
    }
    Ok(value + sample_gaussian(sigma))
}

// TODO: sample gaussian noise!
fn sample_gaussian(_sigma: f64) -> f64 {
    2.
}


pub fn make_base_gaussian(
    input_domain: Domain, sigma: Scalar
) -> Result<Measurement, Error> {

    let sigma = sigma.to_numeric()?;
    let sigma_2 = sigma.clone();

    if let Domain::Scalar(domain) = input_domain.clone() {

        if domain.may_have_nullity {
            return Err(Error::PotentialNullity)
        }
        let (lower, upper) = domain.nature.to_numeric()?.bounds();
        if lower.is_none() { return Err(Error::UnknownBound) }
        if upper.is_none() { return Err(Error::UnknownBound) }
    } else {
        return Err(Error::InvalidDomain)
    }

    Ok(Measurement {
        input_domain,
        privacy_relation: Box::new(move |in_dist: &DataDistance, out_dist: &PrivacyDistance| {

            let in_dist = if let DataDistance::L2Sensitivity(in_dist) = in_dist {
                in_dist
            } else {
                return Err(Error::DistanceMismatch)
            };

            match out_dist {
                PrivacyDistance::Approximate(epsilon, delta) => {
                    let epsilon: f64 = apply_numeric_scalar!(to_f64, epsilon.clone())?;
                    let delta: f64 = apply_numeric_scalar!(to_f64, delta.clone())?;

                    apply_numeric_scalar!(relation_gaussian_mechanism,
                        in_dist.clone(), sigma.clone(); epsilon, delta)
                },
                _ => Err(Error::NotImplemented)
            }
        }),
        // issue: how to differentiate between calls out to different execution environments
        function: Box::new(move |data: Data| match data {
            Data::Value(value) => {
                let value: f64 = value.to_scalar()?.to_finite_float()?.to_f64()?
                    .to_f64().ok_or_else(|| Error::AtomicMismatch)?;
                let sigma: f64 = apply_numeric_scalar!(to_f64, sigma_2.clone())?;
                let release: f64 = gaussian_mechanism(value, sigma)?;

                Ok(Data::Value(Value::Scalar(Scalar::from(release))))
            },
            _ => Err(Error::NotImplemented)
        })
    })
}
