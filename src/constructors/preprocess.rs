use crate::base::{Domain, Scalar, ScalarDomain, NumericScalar, NumericDomain, Data};
use crate::Transformation;
use crate::metric::DataDistance;

pub fn make_clamp_numeric(input_domain: Domain, lower: Scalar, upper: Scalar) -> Result<Transformation, crate::Error> {

    let clamp_atomic_domain = |atomic_type: &Domain| -> Result<Domain, crate::Error> {
        let ScalarDomain { may_have_nullity, nature } = atomic_type.scalar()?.clone();

        let lower = lower.clone().numeric()?;
        let upper = upper.clone().numeric()?;

        let NumericDomain {
            lower: prior_lower, upper: prior_upper
        } = nature.numeric()?;

        let lower = prior_lower.as_ref()
            .map(|prior_lower| lower.max(&prior_lower))
            .transpose()?.unwrap_or(lower);

        let upper = prior_upper.as_ref()
            .map(|prior_upper| upper.min(&prior_upper))
            .transpose()?.unwrap_or(upper);

        Domain::numeric_scalar(Some(lower), Some(upper), may_have_nullity)
    };

    let output_domain = match input_domain.clone() {
        Domain::Vector(mut domain) => {
            domain.atomic_type = Box::new(clamp_atomic_domain(domain.atomic_type.as_ref())?);
            Domain::Vector(domain)
        }
        Domain::Dataframe(mut domain) => {
            for atomic_domain in domain.columns.values_mut() {
                clamp_atomic_domain(atomic_domain).map(|r| *atomic_domain = r)?;
            }
            Domain::Dataframe(domain)
        }
        Domain::Matrix(mut domain) => {
            domain.atomic_type = Box::new(clamp_atomic_domain(domain.atomic_type.as_ref())?);
            Domain::Matrix(domain)
        }
        _ => return Err(crate::Error::Raw("invalid input domain"))
    };

    Ok(Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |in_dist: &DataDistance, out_dist: &DataDistance| in_dist <= out_dist),
        // issue: how to differentiate between calls out to different execution environments
        function: Box::new(move |_data: Data| Err(crate::Error::NotImplemented)),
        hint: Some(Box::new(move |_in_dist: &DataDistance, out_dist: &DataDistance| out_dist.clone())),
    })
}


pub fn make_impute_numeric(
    input_domain: &Domain, lower: NumericScalar, upper: NumericScalar,
) -> Result<Transformation, crate::Error> {
    if lower > upper {
        return Err(crate::Error::Raw("lower may not be less than upper"))
    }

    // function that applies impute transformation to atomic type
    let impute_atomic_domain = |atomic_type: &Domain| -> Result<Domain, crate::Error> {

        // atomic type must be a scalar
        let nature = atomic_type.scalar()?.clone().nature;

        // retrieve lower and upper bounds for the data domain
        let NumericDomain {
            lower: prior_lower, upper: prior_upper
        } = nature.numeric()?.clone();

        // if lower bound on the input domain exists, then potentially widen it or return none
        let lower = Some(prior_lower
            .map(|prior_lower| prior_lower.max(&lower))
            .transpose()?.unwrap_or(lower.clone()));

        // if upper bound on the input domain exists, then potentially widen it or return none
        let upper = Some(prior_upper
            .map(|prior_upper| prior_upper.min(&upper))
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
        Domain::Dataframe(mut domain) => {
            domain.columns.values_mut().try_for_each(|atomic_domain|
                impute_atomic_domain(atomic_domain).map(|r| *atomic_domain = r))?;
            Domain::Dataframe(domain)
        }
        Domain::Matrix(mut domain) => {
            // apply imputation transformation to the atomic domain
            domain.atomic_type = Box::new(impute_atomic_domain(domain.atomic_type.as_ref())?);
            Domain::Matrix(domain)
        }
        _ => return Err(crate::Error::InvalidDomain)
    };

    Ok(Transformation {
        input_domain: input_domain.clone(),
        output_domain,
        stability_relation: Box::new(move |d_in: &DataDistance, d_out: &DataDistance| d_in <= d_out),
        function: Box::new(move |_data| Err(crate::Error::NotImplemented)),
        hint: None,
    })
}

#[cfg(test)]
pub mod test_impute_numeric {
    use crate::base::{Domain, NumericScalar, VectorDomain};
    use crate::constructors::preprocess::make_impute_numeric;

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