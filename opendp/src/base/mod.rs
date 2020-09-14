use crate::base::value::Value;

#[macro_use]
pub mod value;
pub mod domain;
pub mod metric;
pub (crate) mod functions;


#[derive(Clone, Debug)]
pub enum Data {
    Pointer(i64),
    Value(Value),
}
