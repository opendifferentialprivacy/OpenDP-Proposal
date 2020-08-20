use crate::metric::{DataMetric, PrivacyMeasure, DataDistance, PrivacyDistance};
use crate::domain::{DataDomain, VectorDomain, Scalar, AtomicDomain, IntDomain, I64Domain};
use crate::Error;
use std::ops::{Mul, Add};
use std::cmp::Ordering;

use num::


#[derive(Clone, Debug)]
pub enum Data {
    Pointer(i64),
    // Literal(Value)
}

macro_rules! apply_numeric {
    ($bound:expr) => {
        match
    }
}

pub enum ValueScalar {
    F32(f32),
    F64(f64),
    Maybe(Option<NonNullScalar>),
    Must(NonNullScalar)
}

pub enum NonNullScalar {
    Bool(bool),
    String(String),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

pub enum ValueVector {
    F64(Vec<f64>),
    F32(Vec<f32>),
    I128(Vec<i128>),
    I64(Vec<i64>),
    I32(Vec<i32>),
    I16(Vec<i16>),
    I8(Vec<i8>),
    U128(Vec<u128>),
    U64(Vec<u64>),
    U32(Vec<u32>),
    U16(Vec<u16>),
    U8(Vec<u8>),
    Bool(Vec<bool>),
    String(Vec<String>),
}

impl PartialEq<ValueScalar> for ValueScalar {
    fn eq(&self, other: &ValueScalar) -> bool {
        // TODO
        true
    }
}

impl PartialOrd<ValueScalar> for ValueScalar {
    fn partial_cmp(&self, other: &ValueScalar) -> Option<Ordering> {
        // TODO
        Some(Ordering::Equal)
    }
}

#[derive(PartialEq, Clone)]
pub enum DataDomain {
    Scalar(AtomicDomain),
    Vector(VectorDomain<i64>)
}

#[derive(PartialEq, Clone)]
pub struct VectorDomain<TLength: PartialEq + Clone> {
    pub atomic_type: Box<DataDomain>,
    pub is_nonempty: bool,
    pub length: Option<TLength>
}

#[derive(PartialEq, Clone)]
pub enum AtomicDomain {
    Continuous(ContinuousDomain),
    Categorical(CategoricalDomain),
    Bool
}

#[derive(PartialEq, Clone)]
pub struct ContinuousDomain {
    lower: Option<ValueScalar>,
    upper: Option<ValueScalar>
}

impl ContinuousDomain {
    fn new(lower: Option<ValueScalar>, upper: Option<ValueScalar>) -> Result<ContinuousDomain, Error> {
        if let (Some(l), Some(u)) = (&lower, &upper) {
            if std::mem::discriminant(l) != std::mem::discriminant(u) {
                return Err(Error::DomainMismatch)
            }
        }

        // check that lower/upper are not null or optional
        macro_rules! is_continuous {
            ($bound:expr) => {
                match bound {
                    ValueScalar::F64(v) => if !v.is_finite() {return Err(Error::DomainMismatch)},
                    ValueScalar::F32(v) => if !v.is_finite() {return Err(Error::DomainMismatch)},

                    ValueScalar::I8(_) => (),
                    ValueScalar::I16(_) => (),
                    ValueScalar::I32(_) => (),
                    ValueScalar::I64(_) => (),
                    ValueScalar::I128(_) => (),
                    ValueScalar::U8(_) => (),
                    ValueScalar::U16(_) => (),
                    ValueScalar::U32(_) => (),
                    ValueScalar::U64(_) => (),
                    ValueScalar::U128(_) => (),
                    _ => return Err(Error::DomainMismatch)
                }
            }
        }
        is_continuous!(lower);
        is_continuous!(upper);

        Ok(ContinuousDomain {lower, upper})
    }
}

#[derive(PartialEq, Clone)]
pub struct CategoricalDomain(ValueVector);
impl CategoricalDomain {
    fn new(categories: ValueVector) -> Result<CategoricalDomain, Error> {
        // check that categories are not optional or float (must have Eq trait bound)
        match categories {
            ValueScalar::Bool(_) => (),
            ValueScalar::String(_) => (),

            ValueScalar::I8(_) => (),
            ValueScalar::I16(_) => (),
            ValueScalar::I32(_) => (),
            ValueScalar::I64(_) => (),
            ValueScalar::I128(_) => (),
            ValueScalar::U8(_) => (),
            ValueScalar::U16(_) => (),
            ValueScalar::U32(_) => (),
            ValueScalar::U64(_) => (),
            ValueScalar::U128(_) => (),
            _ => return Err(Error::DomainMismatch)
        }
        Ok(CategoricalDomain(categories))
    }
}



