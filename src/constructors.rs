use crate::{Error, InteractiveMeasurement, Measurement, Queryable, Transformation};
use crate::base::{Data, Domain, NumericDomain, NumericScalar, Scalar, ScalarDomain};
use crate::metric::{DataDistance, PrivacyDistance};

pub fn make_adaptive_composition(
    input_domain: Domain,
    input_distance: DataDistance,
    privacy_budget: PrivacyDistance,
) -> InteractiveMeasurement {
    InteractiveMeasurement {
        input_domain: input_domain.clone(),
        input_distance: input_distance.clone(),
        privacy_loss: privacy_budget.clone(),
        function: Box::new(move |data: Data| -> Queryable<(Data, PrivacyDistance)> {
            let input_domain = input_domain.clone();
            Queryable {
                state: (data, privacy_budget.clone()),
                eval: Box::new(move |
                    // query
                    query: Measurement,
                    privacy_loss: PrivacyDistance,
                    // state
                    (data, privacy_budget): &(Data, PrivacyDistance)
                | -> (Result<Data, Error>, (Data, PrivacyDistance)) {
                    if query.input_domain != input_domain.clone() {
                        return (Err(Error::DomainMismatch), (data.clone(), privacy_budget.clone()))
                    }
                    if privacy_budget < &privacy_loss {
                        (Err(Error::InsufficientBudget), (data.clone(), privacy_budget.clone()))
                    } else {
                        match privacy_budget - &privacy_loss {
                            Ok(new_budget) => ((query.function)(data.clone()), (data.clone(), new_budget)),
                            Err(e) => (Err(e), (data.clone(), privacy_budget.clone()))
                        }
                    }
                })
            }
        })
    }
}


// issue: state is hardcoded, not generic
pub fn postprocess(
    interactive_measurement: InteractiveMeasurement,
    queryable_map: Box<dyn Fn(Queryable<(Data, PrivacyDistance)>) -> Queryable<(Data, PrivacyDistance)>>
) -> InteractiveMeasurement {
    let function = interactive_measurement.function;
    InteractiveMeasurement {
        input_domain: interactive_measurement.input_domain,
        input_distance: interactive_measurement.input_distance,
        privacy_loss: interactive_measurement.privacy_loss,
        function: Box::new(move |data: Data| {
            let queryable_inner = (*function)(data);
            queryable_map(queryable_inner)
        })
    }
}

pub fn make_row_transform(
    input_domain: Domain,
    output_domain: Domain,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>,
    hint: Option<Box<dyn Fn(&DataDistance, &DataDistance) -> DataDistance>>,
) -> Transformation {
    Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |_input_distance: &DataDistance, _output_distance: &DataDistance| -> bool { true }),
        function,
        hint,
    }
}



pub fn make_clamp(input_domain: Domain, lower: Scalar, upper: Scalar) -> Result<Transformation, Error> {

    let clamp_atomic_domain = |atomic_type: &Domain| -> Result<Domain, crate::Error> {
        let ScalarDomain { may_have_nullity, nature } = atomic_type.scalar()?.clone();

        let lower = lower.clone().numeric()?;
        let upper = upper.clone().numeric()?;

        let lower = Some(nature.numeric()?.lower.as_ref()
            .map(|prior_lower| lower.max(&prior_lower))
            .transpose()?.unwrap_or(lower));

        let upper = Some(nature.numeric()?.upper.as_ref()
            .map(|prior_upper| upper.min(&prior_upper))
            .transpose()?.unwrap_or(upper));

        Domain::numeric_scalar(lower, upper, may_have_nullity)
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
        _ => return Err(Error::Raw("invalid input domain"))
    };

    Ok(Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |in_dist: &DataDistance, out_dist: &DataDistance| in_dist <= out_dist),
        // issue: how to differentiate between calls out to different execution environments
        function: Box::new(move |data: Data| Ok(data)),
        hint: Some(Box::new(move |_in_dist: &DataDistance, out_dist: &DataDistance| out_dist.clone())),
    })
}

pub fn make_tt_chain(
    trans_2: Transformation, trans_1: Transformation,
    hint: Option<Box<dyn Fn(&DataDistance, &DataDistance) -> DataDistance>>,
    // hint_hint: Option<Box<dyn Fn(&>>
) -> Result<Transformation, crate::Error> {
    if trans_2.input_domain != trans_1.output_domain {
        return Err(crate::Error::DomainMismatch)
    }

    let Transformation {
        input_domain: trans_1_input_domain,
        stability_relation: trans_1_stability_relation,
        function: trans_1_function,
        ..
    } = trans_1;

    let Transformation {
        output_domain: trans_2_output_domain,
        stability_relation: trans_2_stability_relation,
        function: trans_2_function,
        ..
    } = trans_2;

    Ok(Transformation {
        input_domain: trans_1_input_domain,
        output_domain: trans_2_output_domain,
        stability_relation: Box::new(move |d_in: &DataDistance, d_out: &DataDistance| {
            let d_mid = (hint.as_ref().unwrap())(d_in, d_out);
            (trans_2_stability_relation)(&d_mid, d_out) && (trans_1_stability_relation)(d_in, &d_mid)
        }),
        function: Box::new(move |data: Data| (trans_2_function)((trans_1_function)(data)?)),
        hint: None,
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
        function: Box::new(move |data| {
            // TODO: apply lower and upper
            Ok(data)
        }),
        hint: None,
    })
}

#[cfg(test)]
pub mod test_impute_numeric {
    use crate::base::{Domain, NumericScalar, VectorDomain};
    use crate::constructors::make_impute_numeric;

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
            panic!("TODO")
        }
    }
}


// fn make_sum(input_domain: Domain) -> Result<Transformation, Error> {
//
//     let output_domain = match &input_domain {
//         Domain::Vector(VectorDomain {
//                            atomic_type,
//                            is_nonempty,
//                            length
//                        }) => {
//
//             let atomic_type: &ScalarDomain = atomic_type.scalar()?;
//
//             let lower = Some(match (&atomic_type.nature.numeric()?.lower, length) {
//                 (Some(lower), Some(length)) => lower * length,
//                 _ => None
//             });
//
//             let upper = Some(match (&atomic_type.nature.numeric()?.upper, length) {
//                 (Some(upper), Some(length)) => upper * length,
//                 _ => None
//             });
//             Domain::numeric_scalar(lower, upper, atomic_type.may_have_nullity)?
//         },
//         _ => return Err(Error::Raw("invalid input domain"))
//     };
//
//     Ok(Transformation {
//         input_domain,
//         output_domain,
//         stability_relation: Box::new(move |in_dist: DataDistance, out_dist: DataDistance| in_dist <= out_dist),
//         function: Box::new(move |data: Data| Ok(data))
//     })
// }


// fn make_noisy_sum_function(
//     input_domain: DataDomain,
//     function: Box<dyn Fn(Data) -> Result<Data, Error>>,
//     L: f64, U: f64, privacy_distance: PrivacyDistance
// ) -> Measurement {
//
// }