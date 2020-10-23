
// use indexmap::map::IndexMap;
// use noisy_float::types::{R32, R64};
//
// use crate::Error;

// #[derive(Clone, Debug)]
// pub enum Value {
//     Scalar(Scalar),
//     Vector(Vector),
//     Dataframe(Dataframe),
// }
//
// #[derive(Clone, Debug)]
// // TODO: Value -> Vector
// pub struct Dataframe(pub IndexMap<String, Value>);
// pub struct Partition(pub Vec<Value>);

// #[derive(PartialEq, Clone, Debug)]
// pub enum Scalar {
//     Bool(bool),
//     OptionBool(Option<bool>),
//
//     String(String),
//     OptionString(Option<String>),
//
//     U8(u8),
//     U16(u16),
//     U32(u32),
//     U64(u64),
//     U128(u128),
//
//     I8(i8),
//     I16(i16),
//     I32(i32),
//     I64(i64),
//     I128(i128),
//
//     OptionI8(Option<i8>),
//     OptionI16(Option<i16>),
//     OptionI32(Option<i32>),
//     OptionI64(Option<i64>),
//     OptionI128(Option<i128>),
//
//     OptionU8(Option<u8>),
//     OptionU16(Option<u16>),
//     OptionU32(Option<u32>),
//     OptionU64(Option<u64>),
//     OptionU128(Option<u128>),
//
//     R32(R32),
//     R64(R64),
//
//     F32(f32),
//     F64(f64),
// }

// #[derive(PartialEq, Clone, Debug)]
// pub enum Vector {
//     Bool(Vec<bool>),
//     OptionBool(Vec<Option<bool>>),
//
//     String(Vec<String>),
//     OptionString(Vec<Option<String>>),
//
//     I8(Vec<i8>),
//     I16(Vec<i16>),
//     I32(Vec<i32>),
//     I64(Vec<i64>),
//     I128(Vec<i128>),
//
//     OptionI8(Vec<Option<i8>>),
//     OptionI16(Vec<Option<i16>>),
//     OptionI32(Vec<Option<i32>>),
//     OptionI64(Vec<Option<i64>>),
//     OptionI128(Vec<Option<i128>>),
//
//     U8(Vec<u8>),
//     U16(Vec<u16>),
//     U32(Vec<u32>),
//     U64(Vec<u64>),
//     U128(Vec<u128>),
//
//     OptionU8(Vec<Option<u8>>),
//     OptionU16(Vec<Option<u16>>),
//     OptionU32(Vec<Option<u32>>),
//     OptionU64(Vec<Option<u64>>),
//     OptionU128(Vec<Option<u128>>),
//
//     R32(Vec<R32>),
//     R64(Vec<R64>),
//
//     F32(Vec<f32>),
//     F64(Vec<f64>),
// }