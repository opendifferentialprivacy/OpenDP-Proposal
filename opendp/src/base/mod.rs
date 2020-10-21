#[macro_use]
pub mod value;
pub mod domain;
pub mod metric;
pub (crate) mod functions;
pub (crate) mod traits;

use opendp_derive::AutoGet;
use crate::Error;

#[derive(Clone, Debug)]
pub enum Data<T> {
    Pointer(i64),
    Value(T),
    // Value(Value),
}

impl<T> Data<T> {
    pub fn as_pointer(&self) -> Result<&i64, Error> {
        if let Data::Pointer(v) = self {
            Ok(v)
        } else {
            Err(Error::AtomicMismatch)
        }
    }
    pub fn to_pointer(self) -> Result<i64, Error> {
        if let Data::Pointer(v) = self {
            Ok(v)
        } else {
            Err(Error::AtomicMismatch)
        }
    }
    pub fn as_value(&self) -> Result<&T, Error> {
        if let Data::Value(v) = self {
            Ok(v)
        } else {
            Err(Error::AtomicMismatch)
        }
    }
    pub fn to_value(self) -> Result<T, Error> {
        if let Data::Value(v) = self {
            Ok(v)
        } else {
            Err(Error::AtomicMismatch)
        }
    }
}