use crate::example_8_5::metric::{DataMetric, PrivacyMeasure, DataDistance, PrivacyDistance};
use crate::example_8_5::domain::{DataDomain, Vector, Scalar, AtomicDomain, IntDomain, I64Domain};
use std::ops::{Mul, Add};

mod domain;
mod metric;

type Error = &'static str;


#[derive(Clone, Debug)]
enum Data {
    Pointer(i64),
    // Literal(Value)
}

struct Transformation {
    input_domain: DataDomain,
    output_domain: DataDomain,
    stability_relation: Box<dyn Fn(DataDistance, DataDistance) -> bool>,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>
}

struct Measurement {
    input_metric: DataMetric,
    input_domain: DataDomain,
    output_measure: PrivacyMeasure,
    privacy_relation: Box<dyn Fn(DataDistance, PrivacyDistance) -> bool>,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>
}

struct InteractiveMeasurement {
    input_domain: DataDomain,
    input_distance: DataDistance,
    privacy_loss: PrivacyDistance,
    function: Box<dyn Fn(Data) -> Queryable<(Data, PrivacyDistance)>>
}

struct Queryable<S> {
    state: S,
    eval: Box<dyn Fn(Measurement, PrivacyDistance, &S) -> (Result<Data, Error>, S)>
}
impl<S> Queryable<S> {
    fn query(&mut self, measurement: Measurement, privacy_loss: PrivacyDistance) -> Result<Data, Error> {
        let (response, state) = (self.eval)(measurement, privacy_loss, &self.state);
        self.state = state;
        return response
    }
}

fn make_adaptive_composition(
    input_domain: DataDomain,
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
                        return (Err("domain mismatch"), (data.clone(), privacy_budget.clone()))
                    }
                    if privacy_budget < &privacy_loss {
                        (Err("insufficient budget"), (data.clone(), privacy_budget.clone()))
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
fn postprocess(
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


fn make_row_transform(input_domain: DataDomain, output_domain: DataDomain, function: Box<dyn Fn(Data) -> Result<Data, Error>>) -> Transformation {
    Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |input_distance: DataDistance, output_distance: DataDistance| -> bool { true }),
        function,
    }
}

// fn make_base_gaussian() -> Measurement {
//     Measurement {
//         input_metric: (),
//         input_domain: (),
//         output_measure: (),
//         privacy_relation: Box::new(()),
//         function: Box::new(())
//     }
// }

enum AtomicValue {
    F64(f64),
    F32(f32),
    I128(i128),
    I64(i64),
    I32(i32),
    I16(i16),
    I8(i8),
    U128(u128),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
    Bool(bool),
    String(String),
}

fn make_clamp(input_domain: DataDomain, lower: Value, upper: Value) -> Result<Transformation, Error> {
    let mut output_domain = DataDomain::Vector(match &input_domain {
        DataDomain::Vector(Vector {
                               atomic_type,
                               is_empty,
                               length
                           }) => {

            let prior_lower: Value = atomic_type.get_lower();
            let lower = lower.partial_max(prior_lower)?;

            // let (prior_lower, prior_upper) = match *atomic_type.clone() {
            //     DataDomain::Scalar(Scalar(AtomicDomain::Int(IntDomain::I64(I64Domain {lower, upper, categories})))) =>
            //         (lower, upper),
            //     _ => return Err("invalid atomic type")
            // };
            // let lower = match prior_lower {
            //     Some(prior_lower) => prior_lower.max(lower),
            //     None => lower
            // };
            // let upper = match prior_upper {
            //     Some(prior_upper) => prior_upper.max(upper),
            //     None => upper
            // };
            Vector {
                atomic_type: Box::new(DataDomain::Scalar(Scalar(AtomicDomain::Int(IntDomain::I64(I64Domain {
                    lower: Some(lower),
                    upper: Some(upper),
                    categories: None,
                }))))),
                is_empty: *is_empty,
                length: length.clone(),
            }
        },
        _ => return Err("invalid input domain")
    });
    Ok(Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |in_dist: DataDistance, out_dist| in_dist <= out_dist),
        // issue: how to differentiate between calls out to different execution environments
        function: Box::new(move |data| Ok(data))
    })
}

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