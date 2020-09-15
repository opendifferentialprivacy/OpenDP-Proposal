use std::ops::Mul;

use crate::{Error, Transformation};
use crate::base::Data;
use crate::base::domain::{Domain, ScalarDomain, VectorDomain};
use crate::base::metric::DataDistance;
use crate::base::value::*;
use num::NumCast;
use num::cast::cast;
use std::iter::Sum;


fn mul_constant<T>(l: T, r: usize) -> Result<T, Error>
    where T: Mul<Output=T> + NumCast {
    Ok(l.mul(cast::<usize, T>(r).ok_or_else(|| Error::AtomicMismatch)?))
}

fn sum<T: Sum<T>>(data: Vec<T>) -> Result<T, Error> {
    Ok(data.into_iter().sum())
}

fn make_sum(input_domain: Domain) -> Result<Transformation, Error> {
    let output_domain = match &input_domain {
        Domain::Vector(VectorDomain {
                           atomic_type,
                           is_nonempty: _,
                           length
                       }) => {
            let atomic_type: &ScalarDomain = atomic_type.as_scalar()?;
            let (lower, upper) = atomic_type.nature.as_numeric()?.clone().bounds();

            let lower: Option<NumericScalar> = match (&lower, length) {
                (Some(lower), Some(length)) => Some(apply_numeric_scalar!(mul_constant, lower.clone(); *length)?),
                _ => None
            };

            let upper: Option<NumericScalar> = match (&upper, length) {
                (Some(upper), Some(length)) => Some(apply_numeric_scalar!(mul_constant, upper.clone(); *length)?),
                _ => None
            };
            Domain::numeric_scalar(lower, upper, atomic_type.may_have_nullity)?
        }
        _ => return Err(crate::Error::Raw("invalid input domain"))
    };

    Ok(Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |in_dist: &DataDistance, out_dist: &DataDistance| in_dist <= out_dist),
        function: Box::new(move |data: Data| {
            let data: NumericVector = data.to_value()?.to_vector()?.to_numeric()?;
            apply_numeric_vector!(sum, data).map(|v| Data::Value(Value::Scalar(v)))
        }),
        hint: None,
    })
}