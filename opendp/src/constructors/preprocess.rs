
use crate::{Transformation, Error};
use crate::base::domain::{Domain, ScalarDomain, Interval, Nature, VectorDomain};
use crate::base::value::*;
use crate::base::metric::DataDistance;
// use crate::base::Data;
use crate::base::{functions as fun, Data};
use std::cmp::Ordering;
use opendp_derive::{apply_numeric, apply_option_integer};
use std::ops::{Div, Add};
use num::NumCast;
use num::cast;

fn clamp<T: PartialOrd>(v: T, l: T, u: T) -> Result<T, Error> {
    fun::min(fun::max(v, l)?, u)
}

fn clamp_vec<T: PartialOrd + Clone>(v: Vec<T>, l: T, u: T) -> Result<Vec<T>, Error> {
    v.into_iter().map(|v| clamp(v, l.clone(), u.clone())).collect()
}

pub fn make_clamp_numeric<T>(input_domain: &dyn Domain<T>, lower: T, upper: T) -> Result<Transformation<Vec<T>, Vec<T>>, Error> {

    let clamp_atomic_domain = |atomic_type: &dyn Domain<T>| -> Result<dyn Domain<T>, Error> {
        if let Some()
        let ScalarDomain { ref may_have_nullity, ref nature } = atomic_type.as_scalar()?.clone();

        let (prior_lower, prior_upper) = nature.clone().to_numeric()?.bounds();

        let lower: Scalar = prior_lower.as_ref()
            .map(|prior_lower| fun::max(&lower, &prior_lower))
            .transpose()?.unwrap_or(lower.clone());

        let upper: Scalar = prior_upper.as_ref()
            .map(|prior_upper| fun::min(&upper, &prior_upper))
            .transpose()?.unwrap_or(upper.clone());

        Ok(Box::new(ScalarDomain {
            may_have_nullity: *may_have_nullity,
            nature: Nature::Numeric(Interval::new(Some(lower), Some(upper))?)
        }) as _)
    };

    let output_domain = match input_domain.clone() {
        Domain::Vector(mut domain) => {
            domain.atomic_type = Box::new(clamp_atomic_domain(domain.atomic_type.as_ref())?);
            Domain::Vector(domain)
        }
        _ => return Err(Error::InvalidDomain)
    };

    Ok(Transformation {
        input_domain: Box::new(input_domain),
        output_domain,
        stability_relation: Box::new(move |in_dist: &DataDistance, out_dist: &DataDistance| Ok(in_dist <= out_dist)),
        function: Box::new(move |data: Data<Vec<T>>| match data {
            Data::Value(data) => {
                Ok(Data::Value(clamp_vec(data, lower.clone(), upper.clone())?))
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

pub fn make_impute_integer<T: Clone + PartialOrd>(
    input_domain: &dyn Domain<T>, lower: T, upper: T,
) -> Result<Transformation<Vec<T>, Vec<T>>, Error> {
    if lower > upper {
        return Err(Error::Raw("lower may not be greater than upper".to_string()))
    }

    // function that applies impute transformation to atomic type
    let impute_atomic_domain = |atomic_type: &dyn Domain<T>| -> Result<Box<dyn Domain<T>>, Error> {

        // atomic type must be a scalar
        let nature = atomic_type.as_scalar()?.clone().nature;

        // retrieve lower and upper bounds for the data domain
        let (prior_lower, prior_upper) = nature.to_numeric()?.bounds();

        // if lower bound on the input domain exists, then potentially widen it or return none
        let lower = Some(prior_lower
            .map(|prior_lower| fun::max(&lower, &prior_lower))
            .transpose()?.unwrap_or(lower.clone()));

        // if upper bound on the input domain exists, then potentially widen it or return none
        let upper = Some(prior_upper
            .map(|prior_upper| apply_numeric!(fun::min, &upper: Scalar, &prior_upper: Scalar))
            .transpose()?.unwrap_or(upper.clone()));

        Ok(Box::new(ScalarDomain {
            may_have_nullity: false,
            nature: Nature::Numeric(Interval::new(lower, upper)?)
        }) as _)
    };


    let output_domain = if let Some(mut domain) = input_domain.as_any()
        .downcast_ref::<VectorDomain<Vec<T>, T>>() {

        // apply imputation transformation to the atomic domain
        domain.atomic_type = impute_atomic_domain(domain.atomic_type)?;
        Box::new(domain) as Box<dyn Domain<T>>
    } else { return Err(crate::Error::InvalidDomain) };


    Ok(Transformation {
        input_domain: Box::new(input_domain),
        output_domain: Box::new(output_domain) as Box<dyn Domain<Vec<T>>>,
        stability_relation: Box::new(move |d_in: &DataDistance, d_out: &DataDistance| Ok(d_in <= d_out)),
        function: Box::new(move |data: Data<T>| match data {
            Data::Value(data) => Ok(Data::Value(impute_int_vec(data.to_vector()?,lower_wrap.clone(), upper_wrap.clone())?)),
            _ => Err(Error::NotImplemented)
        })
    })
}

// #[cfg(test)]
// pub mod test_impute_numeric {
//     use crate::constructors::preprocess::make_impute_integer;
//     use crate::base::domain::{Domain, VectorDomain};
//     use crate::base::value::Scalar;
//
//     #[test]
//     fn test_1() {
//         let input_domain = Domain::Vector(VectorDomain {
//             atomic_type: Box::new(Domain::numeric_scalar(None, None, true).unwrap()),
//             is_nonempty: false,
//             length: None,
//         });
//
//         make_impute_integer(
//             &input_domain,
//             2.into(),
//             10.into()).unwrap();
//
//         if !make_impute_integer(
//             &input_domain,
//             20.into(),
//             10.into()).is_err() {
//             panic!("Impute must fail if bounds are unordered.")
//         }
//     }
}