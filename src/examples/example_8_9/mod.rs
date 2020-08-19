use crate::example_8_9::domain::DataDomain;
use crate::example_8_9::metric::{DataDistance, DataMetric, PrivacyMeasure, PrivacyDistance};
use std::fmt::Debug;

pub mod domain;
pub mod metric;
pub mod constructors;
pub mod sugar;

type Error = &'static str;

// TODO: execution
#[derive(Clone, Debug)]
pub enum Data {
    Pointer(i64),
    // Literal(Value)
}

pub struct Transformation<NI, CI, NO, CO>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug,
          NO: PartialOrd + Clone + Debug,
          CO: Eq + Clone + Debug {
    input_domain: DataDomain<NI, CI>,
    input_metric: DataMetric,
    output_domain: DataDomain<NO, CO>,
    output_metric: DataMetric,
    stability_relation: Box<dyn Fn(&DataDistance, &DataDistance) -> bool>,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>,
}

pub struct Measurement<NI, CI>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    input_metric: DataMetric,
    input_domain: DataDomain<NI, CI>,
    output_measure: PrivacyMeasure,
    privacy_relation: Box<dyn Fn(&DataDistance, &PrivacyDistance) -> bool>,
    function: Box<dyn Fn(Data) -> Result<Data, Error>>,
}

impl<NI, CI, NO, CO> Transformation<NI, CI, NO, CO>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug,
          NO: PartialOrd + Clone + Debug,
          CO: Eq + Clone + Debug {
    pub fn input_domain(&self) -> DataDomain<NI, CI> {
        self.input_domain.clone()
    }
    pub fn output_domain(&self) -> DataDomain<NO, CO> {
        self.output_domain.clone()
    }
}
impl<NI, CI> Measurement<NI, CI>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    pub fn input_domain(&self) -> DataDomain<NI, CI> {
        self.input_domain.clone()
    }
}

struct InteractiveMeasurement<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    input_domain: DataDomain<NI, CI>,
    input_distance: DataDistance,
    privacy_loss: PrivacyDistance,
    function: Box<dyn Fn(Data) -> Queryable<NI, CI, S>>,
}

// note: generic over type S
//     when exposing cross-language, will need to create specific instances with predefined types
struct Queryable<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    state: S,
    eval: Box<dyn Fn(Measurement<NI, CI>, PrivacyDistance, &S) -> (Result<Data, Error>, S)>,
}

// change: privacy_loss is an argument
//    because Measurements no longer contain .privacy_loss attribute
impl<NI, CI, S> Queryable<NI, CI, S>
    where NI: PartialOrd + Clone + Debug,
          CI: Eq + Clone + Debug {
    fn query(
        &mut self, measurement: Measurement<NI, CI>, privacy_loss: PrivacyDistance,
    ) -> Result<Data, Error> {
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
                    (data, privacy_budget): &(Data, PrivacyDistance)
                | -> (Result<Data, Error>, (Data, PrivacyDistance)) {

                    if query.input_domain != input_domain.clone() {
                        (Err("domain mismatch"), (data.clone(), privacy_budget.clone()))
                    } else if privacy_budget < &privacy_loss {
                        (Err("insufficient budget"), (data.clone(), privacy_budget.clone()))
                    } else {
                        match privacy_budget - &privacy_loss {
                            Ok(new_budget) => ((query.function)(data.clone()), (data.clone(), new_budget)),
                            Err(e) => (Err(e), (data.clone(), privacy_budget.clone()))
                        }
                    }
                }),
            }
        }),
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
