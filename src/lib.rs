// mod examples;
#![allow(dead_code)]

use crate::base::{Domain, Data};
use crate::metric::{DataDistance, PrivacyDistance, Metric, PrivacyMeasure};

pub mod base;
pub mod metric;
pub mod constructors;



/// All possible errors in the library
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("{1}")]
    Default(#[source] std::io::Error, &'static str),
    #[error("Domain mismatch")]
    DomainMismatch,
    #[error("Insufficient budget")]
    InsufficientBudget,
    #[error("{0}")]
    Raw(&'static str)
}



pub struct Transformation {
    pub(crate) input_domain: Domain,
    pub(crate) output_domain: Domain,
    pub(crate) stability_relation: Box<dyn Fn(DataDistance, DataDistance) -> bool>,
    pub(crate) function: Box<dyn Fn(Data) -> Result<Data, Error>>
}

pub struct Measurement {
    pub(crate) input_metric: Metric,
    pub(crate) input_domain: Domain,
    pub(crate) output_measure: PrivacyMeasure,
    pub(crate) privacy_relation: Box<dyn Fn(DataDistance, PrivacyDistance) -> bool>,
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