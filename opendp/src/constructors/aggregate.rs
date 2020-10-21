use std::iter::Sum;
use std::ops::{Mul, Sub};

use num::cast::cast;
use num::NumCast;

use crate::{Error, Transformation};
use crate::base::Data;
use crate::base::domain::{Domain, ScalarDomain, VectorDomain, Nature, Interval};
use crate::base::functions as fun;
use crate::base::metric::DataDistance;
use crate::base::traits::OmniAbs;
use crate::base::value::{Scalar, Vector, Value};


fn mul_constant<T>(l: T, r: usize) -> Result<T, Error>
    where T: Mul<Output=T> + NumCast {
    cast::<usize, T>(r).map(|r| l.mul(r))
        .ok_or_else(|| Error::AtomicMismatch)
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
pub fn make_sum<C, T>(input_domain: &dyn Domain<C>) -> Result<Transformation<Vec<C>, T>, Error> {
    // destructure the descriptor for the input domain, enforcing that it is a vector
    let VectorDomain {
        atomic_type,
        is_nonempty: _,
        length,
        ..
    } = input_domain.as_any().downcast_ref::<VectorDomain<C, T>>()
        .ok_or_else(|| Error::InvalidDomain)?;

    let atomic_type = atomic_type.as_any().downcast_ref::<ScalarDomain<T>>()
        .ok_or_else(|| Error::InvalidDomain)?;

    let (lower, upper) = atomic_type.nature.as_numeric()?.clone().bounds();

    let output_domain = ScalarDomain {
        may_have_nullity: atomic_type.may_have_nullity,
        nature: Nature::Numeric(Interval::new(
            // derive new lower bound
            match (&lower, length) {
                (Some(lower), Some(length)) =>
                    Some(mul_constant(lower.clone(), *length)?),
                _ => None
            },
            // derive new upper bound
            match (&upper, length) {
                (Some(upper), Some(length)) =>
                    Some(mul_constant(upper.clone(), *length)?),
                _ => None
            }
        )?)
    };

    Ok(Transformation {
        input_domain: Box::new(input_domain),
        output_domain: Box::new(output_domain) as Box<dyn Domain<C>>,
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
                DataDistance::Symmetric(d_in) => relation_symmetric(lower, upper, d_out, d_in),
                DataDistance::Hamming(d_in) => relation_hamming(lower, upper, d_out, d_in),
                _ => Err(Error::DistanceMismatch)
            }
        }),

        // actually execute the algorithm
        function: Box::new(move |data: Data<Vec<T>>| {
            match data {
                Data::Value(data) =>
                    Ok(Data::Value(data.to_vector()?.into_iter().sum())),
                _ => unimplemented!()
            }
        })
    })
}