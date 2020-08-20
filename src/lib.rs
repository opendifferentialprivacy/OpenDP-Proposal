// mod examples;

pub mod base;
pub mod domain;
pub mod metric;
pub mod constructors;



/// All possible errors in the library
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("{1}")]
    Unknown(#[source] std::io::Error, &'static str),
    DomainMismatch,
    InsufficientBudget
}



pub struct Transformation {
    pub(crate) input_domain: DataDomain,
    pub(crate) output_domain: DataDomain,
    pub(crate) stability_relation: Box<dyn Fn(DataDistance, DataDistance) -> bool>,
    pub(crate) function: Box<dyn Fn(Data) -> Result<Data, Error>>
}

pub struct Measurement {
    pub(crate) input_metric: DataMetric,
    pub(crate) input_domain: DataDomain,
    pub(crate) output_measure: PrivacyMeasure,
    pub(crate) privacy_relation: Box<dyn Fn(DataDistance, PrivacyDistance) -> bool>,
    pub(crate) function: Box<dyn Fn(Data) -> Result<Data, Error>>
}

pub struct InteractiveMeasurement {
    pub(crate) input_domain: DataDomain,
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