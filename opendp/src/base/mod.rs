use crate::base::value::Value;

#[macro_use]
pub mod value;
pub mod domain;
pub mod metric;
pub (crate) mod functions;
use opendp_derive::AutoGet;
use crate::Error;

#[derive(Clone, Debug, AutoGet)]
pub enum Data {
    Pointer(i64),
    Value(Value),
}
