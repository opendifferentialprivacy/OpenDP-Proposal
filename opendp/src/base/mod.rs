use crate::base::value::Value;

pub mod value;
pub mod domain;
pub mod metric;
mod apply;


#[derive(Clone, Debug)]
pub enum Data {
    Pointer(i64),
    Value(Value),
}
