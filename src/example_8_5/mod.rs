use crate::example_8_5::metric::{DataMetric, PrivacyMeasure, DataDistance, PrivacyDistance};
use crate::example_8_5::domain::DataDomain;

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
//
// fn make_base_gaussian() -> Measurement {
//     Measurement {
//         input_metric: (),
//         input_domain: (),
//         output_measure: (),
//         privacy_relation: Box::new(()),
//         function: Box::new(())
//     }
// }
//
// fn make_clamp() -> Transformation {
//
// }
//
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