use crate::metric::{DataMetric, PrivacyMeasure, DataDistance, PrivacyDistance};
use crate::domain::{DataDomain, Vector, Scalar, AtomicDomain, IntDomain, I64Domain};
use std::ops::{Mul, Add};
use std::cmp::Ordering;

pub(crate) type Error = &'static str;


#[derive(Clone, Debug)]
pub enum Data {
    Pointer(i64),
    // Literal(Value)
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

pub enum AtomicValue {
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

impl PartialEq<AtomicValue> for AtomicValue {
    fn eq(&self, other: &AtomicValue) -> bool {
        // TODO
        true
    }
}

impl PartialOrd<AtomicValue> for AtomicValue {
    fn partial_cmp(&self, other: &AtomicValue) -> Option<Ordering> {
        // TODO
        Some(Ordering::Equal)
    }
}
