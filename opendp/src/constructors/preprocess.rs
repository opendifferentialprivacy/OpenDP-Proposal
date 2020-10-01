
use crate::{Transformation, Error};
use crate::base::domain::{Domain, ScalarDomain};
use crate::base::value::*;
use crate::base::metric::DataDistance;
// use crate::base::Data;
use crate::base::{functions as fun, Data};
use std::cmp::Ordering;
use opendp_derive::{apply_numeric, apply_option_integer, apply_integer};
use std::ops::{Div, Add};
use num::NumCast;
use num::cast;

fn clamp<T: PartialOrd>(v: T, l: T, u: T) -> Result<T, Error> {
    fun::min(fun::max(v, l)?, u)
}

fn clamp_vec<T: PartialOrd + Clone>(v: Vec<T>, l: T, u: T) -> Result<Vec<T>, Error> {
    v.into_iter().map(|v| clamp(v, l.clone(), u.clone())).collect()
}

pub fn make_clamp_numeric(input_domain: Domain, lower: Scalar, upper: Scalar) -> Result<Transformation, Error> {

    let clamp_atomic_domain = |atomic_type: &Domain| -> Result<Domain, Error> {
        let ScalarDomain { ref may_have_nullity, ref nature } = atomic_type.as_scalar()?.clone();

        let (prior_lower, prior_upper) = nature.clone().to_numeric()?.bounds();

        let lower: Scalar = prior_lower.as_ref()
            .map(|prior_lower| apply_numeric!(fun::max, &lower: Scalar, &prior_lower: Scalar))
            .transpose()?.unwrap_or(lower.clone());

        let upper: Scalar = prior_upper.as_ref()
            .map(|prior_upper| apply_numeric!(fun::min, &upper: Scalar, &prior_upper: Scalar))
            .transpose()?.unwrap_or(upper.clone());

        Domain::numeric_scalar(Some(lower), Some(upper), *may_have_nullity)
    };

    let output_domain = match input_domain.clone() {
        Domain::Vector(mut domain) => {
            domain.atomic_type = Box::new(clamp_atomic_domain(domain.atomic_type.as_ref())?);
            Domain::Vector(domain)
        }
        _ => return Err(Error::InvalidDomain)
    };

    Ok(Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |in_dist: &DataDistance, out_dist: &DataDistance| Ok(in_dist <= out_dist)),
        function: Box::new(move |data: Data| match data {
            Data::Value(data) => {
                let out = apply_numeric!(clamp_vec,
                    data.to_vector()?: Vector,
                    lower.clone(): Scalar,
                    upper.clone(): Scalar)?;
                Ok(Data::Value(Value::Vector(out)))
            },
            _ => Err(Error::NotImplemented)
        })
    })
}


fn wrap_option<T>(v: T) -> Result<Option<T>, Error> { Ok(Some(v)) }

fn impute_int_vec<T: Div<Output=T> + Add<Output=T> + NumCast + Clone>(
    v: Vec<Option<T>>, l: Option<T>, u: Option<T>
) -> Result<Vec<T>, Error> {
    let (l, u, two) = (l.unwrap(), u.unwrap(), cast::<i128, T>(2).unwrap());
    Ok(v.into_iter().map(|v| v.unwrap_or((l.clone() + u.clone()) / two.clone())).collect())
}

pub fn make_impute_integer(
    input_domain: &Domain, lower: Scalar, upper: Scalar,
) -> Result<Transformation, Error> {
    if let Ordering::Greater = apply_integer!(fun::cmp, &lower: Scalar, &upper: Scalar)? {
        return Err(Error::Raw("lower may not be greater than upper".to_string()))
    }

    // check that lower and upper bounds are non-null integers and preprocess for imputation
    // attention: this is tricky
    let lower_wrap: Scalar = apply_integer!(wrap_option, lower.clone(): Scalar)?;
    let upper_wrap: Scalar = apply_integer!(wrap_option, lower.clone(): Scalar)?;

    // function that applies impute transformation to atomic type
    let impute_atomic_domain = |atomic_type: &Domain| -> Result<Domain, Error> {

        // atomic type must be a scalar
        let nature = atomic_type.as_scalar()?.clone().nature;

        // retrieve lower and upper bounds for the data domain
        let (prior_lower, prior_upper) = nature.to_numeric()?.bounds();

        // if lower bound on the input domain exists, then potentially widen it or return none
        let lower = Some(prior_lower
            .map(|prior_lower| apply_numeric!(fun::max, &lower: Scalar, &prior_lower: Scalar))
            .transpose()?.unwrap_or(lower.clone()));

        // if upper bound on the input domain exists, then potentially widen it or return none
        let upper = Some(prior_upper
            .map(|prior_upper| apply_numeric!(fun::min, &upper: Scalar, &prior_upper: Scalar))
            .transpose()?.unwrap_or(upper.clone()));

        Domain::numeric_scalar(lower, upper, false)
    };

    let output_domain = match input_domain.clone() {
        // if input domain is a vector
        Domain::Vector(mut domain) => {
            // apply imputation transformation to the atomic domain
            domain.atomic_type = Box::new(impute_atomic_domain(domain.atomic_type.as_ref())?);
            Domain::Vector(domain)
        }
        _ => return Err(crate::Error::InvalidDomain)
    };

    Ok(Transformation {
        input_domain: input_domain.clone(),
        output_domain,
        stability_relation: Box::new(move |d_in: &DataDistance, d_out: &DataDistance| Ok(d_in <= d_out)),
        function: Box::new(move |data: Data| match data {
            Data::Value(data) => {
                let out = apply_option_integer!(impute_int_vec,
                    data.to_vector()?: Vector,
                    lower_wrap.clone(): Scalar,
                    upper_wrap.clone(): Scalar)?;
                Ok(Data::Value(Value::Vector(out)))
            },
            _ => Err(Error::NotImplemented)
        })
    })
}

#[cfg(test)]
pub mod test_impute_numeric {
    use crate::constructors::preprocess::make_impute_integer;
    use crate::base::domain::{Domain, VectorDomain};
    use crate::base::value::Scalar;

    #[test]
    fn test_1() {
        let input_domain = Domain::Vector(VectorDomain {
            atomic_type: Box::new(Domain::numeric_scalar(None, None, true).unwrap()),
            is_nonempty: false,
            length: None,
        });

        make_impute_integer(
            &input_domain,
            2.into(),
            10.into()).unwrap();

        if !make_impute_integer(
            &input_domain,
            20.into(),
            10.into()).is_err() {
            panic!("Impute must fail if bounds are unordered.")
        }
    }
}