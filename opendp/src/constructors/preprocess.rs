
use crate::{Transformation, Error};
use crate::base::domain::{Domain, ScalarDomain};
use crate::base::value::*;
use crate::base::metric::DataDistance;
use crate::base::Data;
use crate::base::functions as fun;
use std::cmp::Ordering;

pub fn make_clamp_numeric(input_domain: Domain, lower: Scalar, upper: Scalar) -> Result<Transformation, Error> {

    let clamp_atomic_domain = |atomic_type: &Domain| -> Result<Domain, Error> {
        let ScalarDomain { ref may_have_nullity, ref nature } = atomic_type.as_scalar()?.clone();

        let lower = lower.to_numeric()?;
        let upper = upper.to_numeric()?;

        let (prior_lower, prior_upper) = nature.clone().to_numeric()?.bounds();

        let lower: NumericScalar = prior_lower.as_ref()
            .map(|prior_lower| apply_numeric_scalar!(fun::max, &lower, &prior_lower))
            .transpose()?.unwrap_or(lower);

        let upper: NumericScalar = prior_upper.as_ref()
            .map(|prior_upper| apply_numeric_scalar!(fun::min, &upper, &prior_upper))
            .transpose()?.unwrap_or(upper);

        Domain::numeric_scalar(Some(lower), Some(upper), *may_have_nullity)
    };

    let output_domain = match input_domain.clone() {
        Domain::Vector(mut domain) => {
            domain.atomic_type = Box::new(clamp_atomic_domain(domain.atomic_type.as_ref())?);
            Domain::Vector(domain)
        }
        _ => return Err(Error::Raw("invalid input domain"))
    };

    Ok(Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |in_dist: &DataDistance, out_dist: &DataDistance| Ok(in_dist <= out_dist)),
        // issue: how to differentiate between calls out to different execution environments
        function: Box::new(move |_data: Data| Err(crate::Error::NotImplemented))
    })
}


pub fn make_impute_numeric(
    input_domain: &Domain, lower: NumericScalar, upper: NumericScalar,
) -> Result<Transformation, Error> {
    if let Ordering::Greater = apply_numeric_scalar!(fun::cmp, &lower, &upper)? {
        return Err(Error::Raw("lower may not be greater than upper"))
    }

    // function that applies impute transformation to atomic type
    let impute_atomic_domain = |atomic_type: &Domain| -> Result<Domain, Error> {

        // atomic type must be a scalar
        let nature = atomic_type.as_scalar()?.clone().nature;

        // retrieve lower and upper bounds for the data domain
        let (prior_lower, prior_upper) = nature.to_numeric()?.bounds();

        // if lower bound on the input domain exists, then potentially widen it or return none
        let lower = Some(prior_lower
            .map(|prior_lower| apply_numeric_scalar!(fun::max, &lower, &prior_lower))
            .transpose()?.unwrap_or(lower.clone()));

        // if upper bound on the input domain exists, then potentially widen it or return none
        let upper = Some(prior_upper
            .map(|prior_upper| apply_numeric_scalar!(fun::min, &upper, &prior_upper))
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
        function: Box::new(move |_data| Err(crate::Error::NotImplemented)),
    })
}

#[cfg(test)]
pub mod test_impute_numeric {
    use crate::constructors::preprocess::make_impute_numeric;
    use crate::base::domain::{Domain, VectorDomain};
    use crate::base::value::NumericScalar;

    #[test]
    fn test_1() {
        let input_domain = Domain::Vector(VectorDomain {
            atomic_type: Box::new(Domain::numeric_scalar(None, None, true).unwrap()),
            is_nonempty: false,
            length: None,
        });

        make_impute_numeric(
            &input_domain,
            NumericScalar::Int(2u64.into()),
            NumericScalar::Int(10u64.into())).unwrap();

        if !make_impute_numeric(
            &input_domain,
            NumericScalar::Int(20u64.into()),
            NumericScalar::Int(10u64.into())).is_err() {
            panic!("Impute must fail if bounds are unordered.")
        }
    }
}