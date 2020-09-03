// mod examples;
#![allow(dead_code)]

use crate::base::{Domain, Data};
use crate::metric::{DataDistance, PrivacyDistance};

pub mod base;
pub mod metric;
pub mod constructors;
pub mod ffi;



/// All possible errors in the library
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("{1}")]
    Default(#[source] std::io::Error, &'static str),
    #[error("Domain mismatch")]
    DomainMismatch,
    #[error("Atomic Mismatch")]
    AtomicMismatch,
    #[error("Invalid Domain")]
    InvalidDomain,
    #[error("Insufficient budget")]
    InsufficientBudget,
    #[error("Potential Nullity")]
    PotentialNullity,
    #[error("{0}")]
    Raw(&'static str),
    #[error("Not Implemented")]
    NotImplemented
}



pub struct Transformation {
    pub(crate) input_domain: Domain,
    pub(crate) output_domain: Domain,
    pub(crate) stability_relation: Box<dyn Fn(&DataDistance, &DataDistance) -> bool>,
    pub(crate) function: Box<dyn Fn(Data) -> Result<Data, Error>>,
    pub(crate) hint: Option<Box<dyn Fn(&DataDistance, &DataDistance) -> DataDistance>>
}

pub struct Measurement {
    pub(crate) input_domain: Domain,
    pub(crate) privacy_relation: Box<dyn Fn(&DataDistance, &PrivacyDistance) -> bool>,
    pub(crate) function: Box<dyn Fn(Data) -> Result<Data, Error>>
}

pub struct InteractiveMeasurement {
    pub(crate) input_domain: Domain,
    pub(crate) input_distance: DataDistance,
    pub(crate) privacy_loss: PrivacyDistance,
    pub(crate) function: Box<dyn Fn(Data) -> Queryable<(Data, PrivacyDistance)>>
}

pub struct Queryable<S> {
    pub(crate) state: S,
    pub(crate) eval: Box<dyn Fn(Measurement, PrivacyDistance, &S) -> (Result<Data, Error>, S)>
}
impl<S> Queryable<S> {
    fn query(&mut self, measurement: Measurement, privacy_loss: PrivacyDistance) -> Result<Data, Error> {
        let (response, state) = (self.eval)(measurement, privacy_loss, &self.state);
        self.state = state;
        return response
    }
}


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