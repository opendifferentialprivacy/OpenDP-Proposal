use crate::metric::{PrivacyDistance, DataDistance};
use crate::base::{Data, Domain};
use crate::{Error, InteractiveMeasurement, Transformation, Measurement, Queryable};


pub fn make_adaptive_composition(
    input_domain: Domain,
    input_distance: DataDistance,
    privacy_budget: PrivacyDistance
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
    function: Box<dyn Fn(Data) -> Result<Data, Error>>
) -> Transformation {
    Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |_input_distance: DataDistance, _output_distance: DataDistance| -> bool { true }),
        function,
    }
}



// pub fn make_clamp(input_domain: DataDomain, lower: AtomicValue, upper: AtomicValue) -> Result<Transformation, Error> {
//     let mut output_domain = DataDomain::Vector(match &input_domain {
//         DataDomain::Vector(Vector {
//                                atomic_type,
//                                is_empty,
//                                length
//                            }) => {
//
//             let prior_lower: AtomicValue = atomic_type.get_lower();
//             let lower = lower.partial_max(prior_lower)?;
//             let prior_upper: AtomicValue = atomic_type.get_upper();
//             let upper = upper.partial_max(prior_upper)?;
//
//             // let (prior_lower, prior_upper) = match *atomic_type.clone() {
//             //     DataDomain::Scalar(Scalar(AtomicDomain::Int(IntDomain::I64(I64Domain {lower, upper, categories})))) =>
//             //         (lower, upper),
//             //     _ => return Err("invalid atomic type")
//             // };
//             // let lower = match prior_lower {
//             //     Some(prior_lower) => prior_lower.max(lower),
//             //     None => lower
//             // };
//             // let upper = match prior_upper {
//             //     Some(prior_upper) => prior_upper.max(upper),
//             //     None => upper
//             // };
//             Vector {
//                 atomic_type: Box::new(DataDomain::Scalar(Scalar(AtomicDomain::Int(IntDomain::I64(I64Domain {
//                     lower: Some(lower),
//                     upper: Some(upper),
//                     categories: None,
//                 }))))),
//                 is_empty: *is_empty,
//                 length: length.clone(),
//             }
//         },
//         _ => return Err("invalid input domain")
//     });
//     Ok(Transformation {
//         input_domain,
//         output_domain,
//         stability_relation: Box::new(move |in_dist: DataDistance, out_dist| in_dist <= out_dist),
//         // issue: how to differentiate between calls out to different execution environments
//         function: Box::new(move |data| Ok(data))
//     })
// }

// fn make_sum() -> Transformation {
//
// }
//
// fn make_noisy_sum_function(
//     input_domain: DataDomain,
//     function: Box<dyn Fn(Data) -> Result<Data, Error>>,
//     L: f64, U: f64, privacy_distance: PrivacyDistance
// ) -> Measurement {
//
// }