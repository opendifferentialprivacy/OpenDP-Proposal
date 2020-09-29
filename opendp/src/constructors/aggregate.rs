use std::iter::Sum;
use std::ops::{Mul, Sub};

use num::cast::cast;
use num::NumCast;

use crate::{Error, Transformation};
use crate::base::Data;
use crate::base::domain::{Domain, ScalarDomain, VectorDomain};
use crate::base::functions as fun;
use crate::base::metric::DataDistance;
use crate::base::traits::OmniAbs;
use crate::base::value::{Scalar, Vector, Value};
use opendp_derive::{apply_numeric};


fn mul_constant<T>(l: T, r: usize) -> Result<T, Error>
    where T: Mul<Output=T> + NumCast {
    cast::<usize, T>(r).map(|r| l.mul(r))
        .ok_or_else(|| Error::AtomicMismatch)
}

fn sum<T: Sum<T>>(data: Vec<T>) -> Result<T, Error> {
    Ok(data.into_iter().sum())
}

// SYMMETRIC
// d_out >= d_in * sens
fn relation_symmetric<T>(lower: T, upper: T, d_out: T, d_in: &u32) -> Result<bool, Error>
    where T: PartialOrd + OmniAbs + NumCast + Mul<Output=T> {
    let d_in: T = cast::<u32, T>(*d_in).ok_or_else(|| Error::UnsupportedCast)?;
    sensitivity_symmetric(lower, upper)
        .map(|sens| d_out >= d_in * sens)
}

// max(|m|, |M|)
pub fn sensitivity_symmetric<T>(lower: T, upper: T) -> Result<T, Error>
    where T: PartialOrd + OmniAbs {
    fun::max(lower.omni_abs(), upper.omni_abs())
}

// HAMMING
fn relation_hamming<T>(lower: T, upper: T, d_out: T, d_in: &u32) -> Result<bool, Error>
    where T: PartialOrd + Sub<Output=T> + NumCast + Mul<Output=T> {
    let d_in: T = cast::<u32, T>(*d_in).ok_or_else(|| Error::UnsupportedCast)?;
    Ok(d_out >= d_in * sensitivity_hamming(lower, upper))
}

fn sensitivity_hamming<T>(lower: T, upper: T) -> T
    where T: Sub<Output=T> {
    upper.sub(lower)
}

/// constructor to build a sum transformation over a vector
pub fn make_sum(input_domain: Domain) -> Result<Transformation, Error> {

    // destructure the descriptor for the input domain, enforcing that it is a vector
    let VectorDomain {
        atomic_type,
        is_nonempty: _,
        length
    } = input_domain.as_vector()?;

    let atomic_type: &ScalarDomain = atomic_type.as_scalar()?;
    let (lower, upper) = atomic_type.nature.as_numeric()?.clone().bounds();

    // derive new output domain
    let output_domain = Domain::numeric_scalar(
        // derive new lower bound
        match (&lower, length) {
            (Some(lower), Some(length)) =>
                Some(apply_numeric!(mul_constant, lower.clone(): Scalar; *length)?),
            _ => None
        },
        // derive new upper bound
        match (&upper, length) {
            (Some(upper), Some(length)) =>
                Some(apply_numeric!(mul_constant, upper.clone(): Scalar; *length)?),
            _ => None
        },
        atomic_type.may_have_nullity
    )?;

    Ok(Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |d_in: &DataDistance, d_out: &DataDistance| {
            // ensure existence of bounds
            let lower = lower.clone().ok_or_else(|| Error::UnknownBound)?;
            let upper = upper.clone().ok_or_else(|| Error::UnknownBound)?;

            // L1 and L2 sensitivity are the same
            let d_out = match d_out {
                DataDistance::L1Sensitivity(v) => v,
                DataDistance::L2Sensitivity(v) => v,
                _ => return Err(Error::DistanceMismatch)
            }.clone();

            // check relation depending on input distance type
            match d_in {
                DataDistance::Symmetric(d_in) =>
                    apply_numeric!(relation_symmetric,
                        lower: Scalar,
                        upper: Scalar,
                        d_out: Scalar;
                        d_in),
                DataDistance::Hamming(d_in) =>
                    apply_numeric!(relation_hamming,
                        lower: Scalar,
                        upper: Scalar,
                        d_out: Scalar;
                        d_in),
                _ => Err(Error::DistanceMismatch)
            }
        }),

        // actually execute the algorithm
        function: Box::new(move |data: Data| {
            match data {
                Data::Value(data) =>
                    apply_numeric!(sum, data.to_vector()?: Vector).map(|v| Data::Value(Value::Scalar(v))),
                _ => unimplemented!()
            }
        })
    })
}