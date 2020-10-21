// mod examples;
#![allow(dead_code)]

use std::cmp::Ordering;

use crate::base::Data;
use crate::base::domain::Domain;
use crate::base::metric::{DataDistance, PrivacyDistance};

#[macro_use]
pub mod base;
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
    #[error("Privacy Mismatch")]
    PrivacyMismatch,
    #[error("Distance Mismatch")]
    DistanceMismatch,
    #[error("Invalid Domain")]
    InvalidDomain,
    #[error("Unsupported Cast")]
    UnsupportedCast,
    #[error("Overflow")]
    Overflow,
    #[error("Insufficient Budget")]
    InsufficientBudget,
    #[error("Unknown Bound")]
    UnknownBound,
    #[error("Potential Nullity")]
    PotentialNullity,
    #[error("{0}")]
    Raw(String),
    #[error("Not Implemented")]
    NotImplemented
}

pub struct Transformation<I, O> {
    pub(crate) input_domain: Box<dyn Domain<I>>,
    pub(crate) output_domain: Domain<O>,
    pub(crate) stability_relation: Box<dyn Fn(&DataDistance, &DataDistance) -> Result<bool, Error>>,
    pub(crate) function: Box<dyn Fn(Data<I>) -> Result<Data<O>, Error>>,
}

pub struct Measurement<I> {
    pub(crate) input_domain: Domain<I>,
    // pub(crate) input_metric: Metric,
    pub(crate) privacy_relation: Box<dyn Fn(&DataDistance, &PrivacyDistance) -> Result<bool, Error>>,
    pub(crate) function: Box<dyn Fn(Data<I>) -> Result<Data<I>, Error>>
}

impl<I, O> Transformation<I, O> {
    pub fn input_domain(&self) -> Domain<I> {
        self.input_domain.clone()
    }
    pub fn output_domain(&self) -> Domain<O> {
        self.output_domain.clone()
    }
}

impl<I> Measurement<I> {
    pub fn function(
        &self, data: Data<I>, in_dist: &DataDistance, out_dist: &PrivacyDistance
    ) -> Result<Data<I>, Error> {

        if !self.privacy_relation(in_dist, out_dist)? {
            return Err(Error::InsufficientBudget)
        }
        (self.function)(data)
    }

    pub fn privacy_relation(&self, in_dist: &DataDistance, out_dist: &PrivacyDistance) -> Result<bool, Error> {
        (self.privacy_relation)(in_dist, out_dist)
    }
}

pub struct InteractiveMeasurement<I, O> {
    pub(crate) input_domain: Domain<I>,
    pub(crate) input_distance: DataDistance,
    pub(crate) privacy_loss: PrivacyDistance,
    pub(crate) function: Box<dyn Fn(Data<I>) -> Queryable<O, (Data<O>, PrivacyDistance)>>
}

pub struct Queryable<I, S> {
    pub(crate) state: S,
    pub(crate) eval: Box<dyn Fn(Measurement<I>, PrivacyDistance, &S) -> (Result<Data<I>, Error>, S)>
}
impl<I, S> Queryable<I, S> {
    fn query(
        &mut self, measurement: Measurement<I>, privacy_loss: PrivacyDistance
    ) -> Result<Data<I>, Error> {
        let (response, state) = (self.eval)(measurement, privacy_loss, &self.state);
        self.state = state;
        return response
    }
}


pub fn make_adaptive_composition<I: Clone, O>(
    input_domain: Domain<I>,
    input_distance: DataDistance,
    privacy_budget: PrivacyDistance,
) -> InteractiveMeasurement<I, O> {
    InteractiveMeasurement {
        input_domain: input_domain.clone(),
        input_distance: input_distance.clone(),
        privacy_loss: privacy_budget.clone(),
        function: Box::new(move |data: Data<I>| -> Queryable<O, (Data<I>, PrivacyDistance)> {
            let input_domain = input_domain.clone();
            Queryable {
                state: (data, privacy_budget.clone()),
                eval: Box::new(move |
                    // query
                    query: Measurement<I>,
                    privacy_loss: PrivacyDistance,
                    // state
                    (data, privacy_budget): &(Data<I>, PrivacyDistance)
                | -> (Result<Data<I>, Error>, (Data<I>, PrivacyDistance)) {
                    if query.input_domain != input_domain.clone() {
                        return (Err(Error::DomainMismatch), (data.clone(), privacy_budget.clone()))
                    }
                    // check if privacy budget is sufficient
                    match privacy_budget.partial_cmp(&privacy_loss) {
                        // if privacy budget is not comparable with usage, error
                        None => (Err(Error::PrivacyMismatch), (data.clone(), privacy_budget.clone())),
                        // if privacy budget is sufficient, apply the query
                        Some(Ordering::Greater) => match privacy_budget.clone() - privacy_loss {
                            Ok(new_budget) => ((query.function)(data.clone()), (data.clone(), new_budget)),
                            Err(e) => (Err(e), (data.clone(), privacy_budget.clone()))
                        },
                        // if privacy budget is insufficient, error
                        _ => (Err(Error::InsufficientBudget), (data.clone(), privacy_budget.clone()))
                    }
                })
            }
        })
    }
}


// issue: state is hardcoded, not generic
pub fn postprocess<I, O>(
    interactive_measurement: InteractiveMeasurement<I, O>,
    queryable_map: Box<dyn Fn(Queryable<I, (Data<I>, PrivacyDistance)>) -> Queryable<O, (Data<O>, PrivacyDistance)>>
) -> InteractiveMeasurement<I, O> {
    let function = interactive_measurement.function;
    InteractiveMeasurement {
        input_domain: interactive_measurement.input_domain,
        input_distance: interactive_measurement.input_distance,
        privacy_loss: interactive_measurement.privacy_loss,
        function: Box::new(move |data: Data<I>| {
            let queryable_inner = (*function)(data);
            queryable_map(queryable_inner)
        })
    }
}