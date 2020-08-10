use crate::example_8_9::metric::{DataMetric, PrivacyMeasure, DataDistance, PrivacyDistance};
use crate::example_8_9::domain::{DataDomain, AtomicDomain, NumericDomain};
use std::fmt::Debug;

mod domain;
mod metric;

type Error = &'static str;

#[derive(Clone, Debug)]
enum Data {
    Pointer(i64),
    // Literal(Value)
}

struct Transformation<NI, CI, NO, CO>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug,
          NO: PartialOrd + Clone + Debug,
          CO: Eq + Clone + Debug {
    input_domain: DataDomain<NI, CI>,
    output_domain: DataDomain<NO, CO>,
    stability_relation: Box<dyn Fn(DataDistance, DataDistance) -> bool>,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>,
}

struct Measurement<NI, CI>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    input_metric: DataMetric,
    input_domain: DataDomain<NI, CI>,
    output_measure: PrivacyMeasure,
    privacy_relation: Box<dyn Fn(DataDistance, PrivacyDistance) -> bool>,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>,
}

struct InteractiveMeasurement<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    input_domain: DataDomain<NI, CI>,
    input_distance: DataDistance,
    privacy_loss: PrivacyDistance,
    function: Box<dyn Fn(Data) -> Queryable<NI, CI, S>>,
}

struct Queryable<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    state: S,
    eval: Box<dyn Fn(Measurement<NI, CI>, PrivacyDistance, &S) -> (Result<Data, Error>, S)>,
}

impl<NI, CI, S> Queryable<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    fn query(&mut self, measurement: Measurement<NI, CI>, privacy_loss: PrivacyDistance) -> Result<Data, Error> {
        let (response, state) = (self.eval)(measurement, privacy_loss, &self.state);
        self.state = state;
        return response;
    }
}

fn make_adaptive_composition<NI: 'static, CI: 'static>(
    input_domain: DataDomain<NI, CI>,
    input_distance: DataDistance,
    privacy_budget: PrivacyDistance,
) -> InteractiveMeasurement<NI, CI, (Data, PrivacyDistance)>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    InteractiveMeasurement {
        input_domain: input_domain.clone(),
        input_distance: input_distance.clone(),
        privacy_loss: privacy_budget.clone(),
        function: Box::new(move |data: Data| -> Queryable<NI, CI, (Data, PrivacyDistance)> {
            let input_domain = input_domain.clone();
            Queryable {
                state: (data, privacy_budget.clone()),
                eval: Box::new(move |
                    // query
                    query: Measurement<NI, CI>,
                    privacy_loss: PrivacyDistance,
                    // state
                    (data, privacy_budget): &(Data, PrivacyDistance)| -> (Result<Data, Error>, (Data, PrivacyDistance)) {
                    if query.input_domain != input_domain.clone() {
                        return (Err("domain mismatch"), (data.clone(), privacy_budget.clone()));
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

fn postprocess<NI: 'static, CI: 'static, S: 'static>(
    interactive_measurement: InteractiveMeasurement<NI, CI, S>,
    queryable_map: Box<dyn Fn(Queryable<NI, CI, S>) -> Queryable<NI, CI, S>>,
) -> InteractiveMeasurement<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    let function = interactive_measurement.function;
    InteractiveMeasurement {
        input_domain: interactive_measurement.input_domain,
        input_distance: interactive_measurement.input_distance,
        privacy_loss: interactive_measurement.privacy_loss,
        function: Box::new(move |data: Data| {
            let queryable_inner = (*function)(data);
            queryable_map(queryable_inner)
        }),
    }
}


fn make_row_transform<NI, CI, NO, CO>(
    input_domain: DataDomain<NI, CI>,
    output_domain: DataDomain<NO, CO>,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>,
) -> Transformation<NI, CI, NO, CO>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug,
          NO: PartialOrd + Clone + Debug,
          CO: Eq + Clone + Debug {
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

fn make_clamp_numeric<NI, CI>(
    input_domain: DataDomain<NI, CI>,
    lower: NI, upper: NI,
) -> Result<Transformation<NI, CI, NI, CI>, Error>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {

    let output_domain = match &input_domain {
        DataDomain::Vector {
            atomic_type,
            is_nonempty,
            length
        } => {
            // rest/unpack the prior numeric domain descriptors
            let NumericDomain {
                lower: prior_lower, upper: prior_upper, optional
            } = if let AtomicDomain::Numeric(v) = atomic_type { v } else {
                return Err("invalid atomic type");
            };

            // construct a new domain with updated bounds
            DataDomain::Vector {
                length: length.clone(),
                is_nonempty: *is_nonempty,
                atomic_type: AtomicDomain::Numeric(NumericDomain {
                    lower: Some(prior_lower.as_ref()
                        .map(|l| if l < &lower { &lower } else { l })
                        .unwrap_or(&lower).clone()),
                    upper: Some(prior_upper.as_ref()
                        .map(|u| if u > &upper { &upper } else { u })
                        .unwrap_or(&upper).clone()),
                    optional: *optional,
                }),
            }
        }
        _ => return Err("invalid input domain")
    };

    Ok(Transformation {
        input_domain,
        output_domain,
        stability_relation: Box::new(move |in_dist: DataDistance, out_dist| in_dist <= out_dist),
        // issue: how to differentiate between calls out to different execution environments
        function: Box::new(move |data| Ok(data)),
    })
}

// fn make_sum<NI, CI>(
//     input_domain: DataDomain<NI, CI>,
//     lower: NI, upper: NI,
// ) -> Result<Transformation<NI, CI, NI, CI>, Error>
//     where NI: PartialOrd + Clone + Debug,
//           CI: Eq + Clone + Debug {
//
// }
// fn make_noisy_sum_function(
//     input_domain: DataDomain,
//     function: Box<dyn Fn(Data) -> Result<Data, Error>>,
//     L: f64, U: f64, privacy_distance: PrivacyDistance
// ) -> Measurement {
//
// }

