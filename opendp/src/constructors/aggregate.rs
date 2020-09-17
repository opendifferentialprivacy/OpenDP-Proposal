use std::ops::{Mul, Sub};

use crate::{Error, Transformation};
use crate::base::Data;
use crate::base::domain::{Domain, ScalarDomain, VectorDomain, Nature, Interval};
use crate::base::metric::DataDistance;
use crate::base::value::*;
use crate::base::functions as fun;
use crate::base::traits::OmniAbs;
use num::{NumCast};
use num::cast::cast;
use std::iter::Sum;


fn mul_constant<T>(l: T, r: usize) -> Result<T, Error>
    where T: Mul<Output=T> + NumCast {
    cast::<usize, T>(r).map(|r| l.mul(r))
        .ok_or_else(|| Error::AtomicMismatch)
}

fn sum<T: Sum<T>>(data: Vec<T>) -> Result<T, Error> {
    Ok(data.into_iter().sum())
}

// SYMMETRIC
fn relation_symmetric<T: PartialOrd + OmniAbs + NumCast + Mul<Output=T>>(
    lower: T, upper: T, d_out: T, d_in: &u32
) -> Result<bool, Error> {
    let d_in: T = cast::<u32, T>(*d_in).ok_or_else(|| Error::UnsupportedCast)?;
    sensitivity_symmetric(lower, upper)
        .map(|sens| d_out >= d_in * sens)
}

fn sensitivity_symmetric<T: PartialOrd + OmniAbs>(
    lower: T, upper: T
) -> Result<T, Error> {
    fun::max(lower.omni_abs(), upper.omni_abs())
}

// HAMMING
fn relation_hamming<T>(lower: T, upper: T, d_out: T, d_in: &u32) -> Result<bool, Error>
    where T: PartialOrd + Sub<Output=T> + NumCast + Mul<Output=T> {
    let d_in: T = cast::<u32, T>(*d_in).ok_or_else(|| Error::UnsupportedCast)?;
    Ok(d_out >= d_in * sensitivity_hamming(lower, upper))
}

fn sensitivity_hamming<T>(lower: T, upper: T) -> T
    where T: PartialOrd + Sub<Output=T> + NumCast {
    upper.sub(lower)
}

fn make_sum(input_domain: Domain) -> Result<Transformation, Error> {

    // destructure the descriptor for the input domain, enforcing that it is a vector
    let VectorDomain {
        atomic_type,
        is_nonempty: _,
        length
    } = match &input_domain {
        Domain::Vector(vector_domain) => vector_domain,
        _ => return Err(Error::InvalidDomain)
    };

    let atomic_type: &ScalarDomain = atomic_type.as_scalar()?;
    let (lower, upper) = atomic_type.nature.as_numeric()?.clone().bounds();

    // derive new output domain
    let output_domain = Domain::Scalar(ScalarDomain {
        may_have_nullity: atomic_type.may_have_nullity,
        nature: Nature::Numeric(Interval::new(
            // derive new lower bound
            match (&lower, length) {
                (Some(lower), Some(length)) =>
                    Some(apply_numeric_scalar!(mul_constant, lower.clone(); *length)?),
                _ => None
            },
            // derive new upper bound
            match (&upper, length) {
                (Some(upper), Some(length)) =>
                    Some(apply_numeric_scalar!(mul_constant, upper.clone(); *length)?),
                _ => None
            })?)
    });

    Ok(Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |d_in: &DataDistance, d_out: &DataDistance| {
            let lower = lower.clone().ok_or_else(|| Error::UnknownBound)?;
            let upper = upper.clone().ok_or_else(|| Error::UnknownBound)?;
            let d_out = match d_out {
                DataDistance::L1Sensitivity(v) => v,
                DataDistance::L2Sensitivity(v) => v,
                _ => return Err(Error::DistanceMismatch)
            }.clone();

            match d_in {
                DataDistance::Symmetric(d_in) =>
                    apply_numeric_scalar!(relation_symmetric, lower, upper, d_out; d_in),
                DataDistance::Hamming(d_in) =>
                    apply_numeric_scalar!(relation_hamming, lower, upper, d_out; d_in),
                _ => Err(Error::DistanceMismatch)
            }
        }),

        // actually execute the algorithm
        function: Box::new(move |data: Data| {
            let data = data.to_value()?.to_vector()?.to_numeric()?;

            let data: NumericVector = apply_numeric_vector!(fun::sort, data)?;

            apply_numeric_vector!(sum, data)
                .map(|v| Data::Value(Value::Scalar(v)))
        })
    })
}