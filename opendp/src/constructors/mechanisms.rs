use crate::base::domain::{ScalarDomain};
use crate::{Error, Measurement};
use num::{NumCast, ToPrimitive, Float};
use num::cast::cast;
use crate::base::Data;
use noisy_float::types::{R64, R32};
use std::fmt::Display;
use crate::base::metric::{Distance, Size};

fn to_f64<T: NumCast + Clone>(v: T) -> Result<f64, Error> {
    cast::<T, R64>(v).ok_or_else(|| Error::UnsupportedCast)?
        .to_f64().ok_or_else(|| Error::UnsupportedCast)
}

fn relation_gaussian_mechanism<T>(
    sensitivity: T, sigma: T, epsilon: T, delta: T
) -> Result<bool, Error>
    where T: PartialOrd + NumCast + Float {

    fn to_<T: NumCast>(v: f64) -> Result<T, Error> {
        cast::<f64, T>(v).ok_or(Error::UnsupportedCast)
    }

    if delta.is_sign_negative() {
        return Err(Error::Raw("delta may not be less than zero".to_string()))
    }
    Ok(epsilon.min(to_::<T>(1.0)?) >= (sensitivity / sigma) * ((to_::<T>(1.25)? / delta).ln() * to_::<T>(2.0)?).sqrt())
}

pub trait NoisyFloat: Float {
    fn sample_gaussian(_sigma: Self) -> Self {Self::zero()}
}
impl NoisyFloat for R32 {}
impl NoisyFloat for R64 {}

pub fn gaussian_mechanism<T: NoisyFloat + Display>(
    value: T,
    sigma: T
) -> Result<T, Error> {
    if sigma.is_sign_negative() || sigma.is_zero() {
        return Err(Error::Raw(format!("sigma ({}) must be positive", sigma)));
    }
    Ok(value + NoisyFloat::sample_gaussian(sigma))
}


pub fn make_base_gaussian<T: 'static + NoisyFloat + Display>(
    input_domain: ScalarDomain<T, T>, sigma: T
) -> Result<Measurement<ScalarDomain<T, T>, T, T, T>, Error> {

    let ScalarDomain {
        may_have_nullity, nature, ..
    } = input_domain.clone();

    if may_have_nullity {
        return Err(Error::PotentialNullity)
    }
    let (lower, upper) = nature.to_numeric()?.bounds();
    if lower.is_none() { return Err(Error::UnknownBound) }
    if upper.is_none() { return Err(Error::UnknownBound) }

    Ok(Measurement {
        input_domain,
        privacy_relation: Box::new(move |in_dist: &Distance<T>, out_dist: &Size<T>| {

            let in_dist = if let Distance::L2Sensitivity(in_dist) = in_dist {
                in_dist
            } else {
                return Err(Error::DistanceMismatch)
            };

            match out_dist {
                Size::Approximate(epsilon, delta) =>
                    relation_gaussian_mechanism(in_dist.clone(), sigma.clone(), epsilon.clone(), delta.clone()),
                _ => Err(Error::NotImplemented)
            }
        }),
        function: Box::new(move |data: Data<T>| match data {
            Data::Value(value) => Ok(Data::Value(gaussian_mechanism(value, sigma)?)),
            _ => Err(Error::NotImplemented)
        })
    })
}
