use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{Mul, Sub};

use num::cast::cast;
use num::NumCast;

use crate::{Error, Transformation};
use crate::base::Data;
use crate::base::domain::{Interval, Nature, ScalarDomain, VectorDomain};
use crate::base::functions as fun;
use crate::base::metric::{Metric, Distance};
use crate::base::traits::OmniAbs;

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


type InputDomain<T> = VectorDomain<Vec<T>, ScalarDomain<T, T>>;
type OutputDomain<T> = ScalarDomain<T, T>;

/// constructor to build a sum transformation over a vector
pub fn make_sum<T>(
    input_domain: InputDomain<T>, input_metric: Metric, output_metric: Metric
) -> Result<Transformation<InputDomain<T>, OutputDomain<T>, u32, T>, Error>
    where T: 'static + Sum + PartialOrd + Clone + Mul<Output=T> + Sub<Output=T> + NumCast + OmniAbs {
    // destructure the descriptor for the input domain, enforcing that it is a vector
    let length = input_domain.length.clone();
    let atomic_type = input_domain.atomic_type.clone();

    let (lower, upper) = atomic_type.nature.as_numeric()?.clone().bounds();

    let output_domain = ScalarDomain {
        may_have_nullity: atomic_type.may_have_nullity,
        nature: Nature::Numeric(Interval::new(
            // derive new lower bound
            match (lower.clone(), length) {
                (Some(lower), Some(length)) =>
                    Some(mul_constant(lower, length)?),
                _ => None
            },
            // derive new upper bound
            match (upper.clone(), length) {
                (Some(upper), Some(length)) =>
                    Some(mul_constant(upper, length)?),
                _ => None
            }
        )?),
        container: PhantomData::<T>
    };

    Ok(Transformation {
        input_domain,
        input_metric,
        output_domain,
        output_metric,
        stability_relation: Box::new(move |d_in: &Distance<u32>, d_out: &Distance<T>| {
            // ensure existence of bounds
            let lower = lower.clone().ok_or_else(|| Error::UnknownBound)?;
            let upper = upper.clone().ok_or_else(|| Error::UnknownBound)?;

            // L1 and L2 sensitivity are the same
            let sensitivity = match d_out {
                Distance::L1Sensitivity(v) => v,
                Distance::L2Sensitivity(v) => v,
                _ => return Err(Error::DistanceMismatch)
            }.clone();

            // check relation depending on input distance type
            match d_in {
                Distance::Symmetric(d_in) => relation_symmetric(lower, upper, sensitivity, d_in),
                Distance::Hamming(d_in) => relation_hamming(lower, upper, sensitivity, d_in),
                _ => Err(Error::DistanceMismatch)
            }
        }),

        // actually execute the algorithm
        function: Box::new(move |data: Data<Vec<T>>| {
            match data {
                Data::Value(data) =>
                    Ok(Data::Value(data.into_iter().sum())),
                _ => unimplemented!()
            }
        })
    })
}