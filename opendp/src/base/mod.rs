use crate::base::value::Value;

pub mod value;
pub mod domain;
pub mod metric;


#[derive(Clone, Debug)]
pub enum Data {
    Pointer(i64),
    Value(Value),
}
