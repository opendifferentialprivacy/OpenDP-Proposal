use std::fmt::Debug;

use indexmap::map::IndexMap;
use noisy_float::types::{R32, R64};

use opendp_derive::{AutoFrom, AutoGet, Apply};
use crate::Error;


#[derive(Clone, Debug, AutoFrom, AutoGet)]
pub enum Value {
    Scalar(Scalar),
    Vector(Vector),
    Dataframe(Dataframe),
}

#[derive(Clone, Debug)]
pub struct Dataframe(pub IndexMap<CategoricalScalar, Value>);


// ~~~~ SCALAR ~~~~`
// TYPES

#[derive(Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum Scalar {
    Bool(bool),
    OptionBool(Option<bool>),

    String(String),
    OptionString(Option<String>),

    #[reapply]
    SignedInt(SignedIntScalar),
    #[reapply]
    OptionSignedInt(OptionSignedIntScalar),

    #[reapply]
    UnsignedInt(UnsignedIntScalar),
    #[reapply]
    OptionUnsignedInt(OptionUnsignedIntScalar),

    #[reapply]
    FiniteFloat(FiniteFloatScalar),
    #[reapply]
    Float(FloatScalar),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum UnsignedIntScalar {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum SignedIntScalar {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum OptionSignedIntScalar {
    I8(Option<i8>),
    I16(Option<i16>),
    I32(Option<i32>),
    I64(Option<i64>),
    I128(Option<i128>),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum OptionUnsignedIntScalar {
    U8(Option<u8>),
    U16(Option<u16>),
    U32(Option<u32>),
    U64(Option<u64>),
    U128(Option<u128>),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum FiniteFloatScalar {
    F32(R32),
    F64(R64),
}

#[derive(PartialEq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum FloatScalar {
    F32(f32),
    F64(f64),
}


// SUBSET TYPES
#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum NumericScalar {
    #[reapply]
    FiniteFloat(FiniteFloatScalar),
    #[reapply]
    SignedInt(SignedIntScalar),
    #[reapply]
    UnsignedInt(UnsignedIntScalar)
}

#[derive(PartialEq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum OptionNumericScalar {
    #[reapply]
    Float(FloatScalar),
    #[reapply]
    OptionSignedInt(OptionSignedIntScalar),
    #[reapply]
    OptionUnsignedInt(OptionUnsignedIntScalar),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum CategoricalScalar {
    Bool(bool),
    String(String),
    #[reapply]
    SignedInt(SignedIntScalar),
    #[reapply]
    UnsignedInt(UnsignedIntScalar),
    #[reapply]
    FiniteFloat(FiniteFloatScalar)
}

// IMPLEMENTATIONS
// specialize to subset type
impl Scalar {
    pub fn to_numeric(self) -> Result<NumericScalar, Error> {
        Ok(match self {
            Scalar::SignedInt(v) => NumericScalar::SignedInt(v),
            Scalar::UnsignedInt(v) => NumericScalar::UnsignedInt(v),
            Scalar::FiniteFloat(v) => NumericScalar::FiniteFloat(v),
            _ => return Err(Error::AtomicMismatch)
        })
    }
    pub fn to_categorical(self) -> Result<CategoricalScalar, Error> {
        Ok(match self {
            Scalar::Bool(v) => CategoricalScalar::Bool(v),
            Scalar::String(v) => CategoricalScalar::String(v),
            Scalar::SignedInt(v) => CategoricalScalar::SignedInt(v),
            Scalar::UnsignedInt(v) => CategoricalScalar::UnsignedInt(v),
            Scalar::FiniteFloat(v) => CategoricalScalar::FiniteFloat(v),
            _ => return Err(Error::AtomicMismatch)
        })
    }
}
// generalize subset types
impl NumericScalar {
    pub fn to_scalar(self) -> Scalar {
        match self {
            NumericScalar::FiniteFloat(v) => Scalar::FiniteFloat(v),
            NumericScalar::SignedInt(v) => Scalar::SignedInt(v),
            NumericScalar::UnsignedInt(v) => Scalar::UnsignedInt(v),
        }
    }
}
impl OptionNumericScalar {
    pub fn to_scalar(self) -> Scalar {
        match self {
            OptionNumericScalar::Float(v) => Scalar::Float(v),
            OptionNumericScalar::OptionSignedInt(v) => Scalar::OptionSignedInt(v),
            OptionNumericScalar::OptionUnsignedInt(v) => Scalar::OptionUnsignedInt(v),
        }
    }
}
impl CategoricalScalar {
    pub fn to_scalar(self) -> Scalar {
        match self {
            CategoricalScalar::Bool(v) => Scalar::Bool(v),
            CategoricalScalar::String(v) => Scalar::String(v),
            CategoricalScalar::SignedInt(v) => Scalar::SignedInt(v),
            CategoricalScalar::UnsignedInt(v) => Scalar::UnsignedInt(v),
            CategoricalScalar::FiniteFloat(v) => Scalar::FiniteFloat(v),
        }
    }
}

// ~~~~ VECTOR ~~~~
// TYPES
#[derive(Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum Vector {
    Bool(Vec<bool>),
    OptionBool(Vec<Option<bool>>),

    String(Vec<String>),
    OptionString(Vec<Option<String>>),

    #[reapply]
    SignedInt(SignedIntVector),
    #[reapply]
    OptionSignedInt(OptionSignedIntVector),

    #[reapply]
    UnsignedInt(UnsignedIntVector),
    #[reapply]
    OptionUnsignedInt(OptionUnsignedIntVector),

    #[reapply]
    FiniteFloat(FiniteFloatVector),
    #[reapply]
    Float(FloatVector),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum SignedIntVector {
    I8(Vec<i8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    I128(Vec<i128>),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum UnsignedIntVector {
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    U64(Vec<u64>),
    U128(Vec<u128>),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum OptionSignedIntVector {
    I8(Vec<Option<i8>>),
    I16(Vec<Option<i16>>),
    I32(Vec<Option<i32>>),
    I64(Vec<Option<i64>>),
    I128(Vec<Option<i128>>),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum OptionUnsignedIntVector {
    U8(Vec<Option<u8>>),
    U16(Vec<Option<u16>>),
    U32(Vec<Option<u32>>),
    U64(Vec<Option<u64>>),
    U128(Vec<Option<u128>>),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum FiniteFloatVector {
    F32(Vec<R32>),
    F64(Vec<R64>),
}

#[derive(PartialEq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum FloatVector {
    F32(Vec<f32>),
    F64(Vec<f64>),
}


// SUBSET TYPES
#[derive(PartialEq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum NumericVector {
    #[reapply]
    FiniteFloat(FiniteFloatVector),
    #[reapply]
    SignedInt(SignedIntVector),
    #[reapply]
    UnsignedInt(UnsignedIntVector),
}

#[derive(PartialEq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum OptionNumericVector {
    #[reapply]
    Float(FloatVector),
    #[reapply]
    OptionSignedInt(OptionSignedIntVector),
    #[reapply]
    OptionUnsignedInt(OptionUnsignedIntVector),
}

#[derive(PartialEq, Eq, Clone, Debug, AutoFrom, AutoGet, Apply)]
pub enum CategoricalVector {
    Bool(Vec<bool>),
    String(Vec<String>),
    #[reapply]
    SignedInt(SignedIntVector),
    #[reapply]
    UnsignedInt(UnsignedIntVector),
    #[reapply]
    FiniteFloat(FiniteFloatVector)
}

// IMPLEMENTATIONS
// specialize to subset type
impl Vector {
    pub fn to_numeric(self) -> Result<NumericVector, Error> {
        Ok(match self {
            Vector::SignedInt(v) => NumericVector::SignedInt(v),
            Vector::UnsignedInt(v) => NumericVector::UnsignedInt(v),
            Vector::FiniteFloat(v) => NumericVector::FiniteFloat(v),
            _ => return Err(Error::AtomicMismatch)
        })
    }
    pub fn to_categorical(self) -> Result<CategoricalVector, Error> {
        Ok(match self {
            Vector::Bool(v) => CategoricalVector::Bool(v),
            Vector::String(v) => CategoricalVector::String(v),
            Vector::SignedInt(v) => CategoricalVector::SignedInt(v),
            Vector::UnsignedInt(v) => CategoricalVector::UnsignedInt(v),
            Vector::FiniteFloat(v) => CategoricalVector::FiniteFloat(v),
            _ => return Err(Error::AtomicMismatch)
        })
    }
}
// generalize subset types
impl NumericVector {
    pub(crate) fn to_vector(self) -> Vector {
        match self {
            NumericVector::FiniteFloat(v) => Vector::FiniteFloat(v),
            NumericVector::SignedInt(v) => Vector::SignedInt(v),
            NumericVector::UnsignedInt(v) => Vector::UnsignedInt(v),
        }
    }
}
impl OptionNumericVector {
    pub(crate) fn to_vector(self) -> Vector {
        match self {
            OptionNumericVector::Float(v) => Vector::Float(v),
            OptionNumericVector::OptionSignedInt(v) => Vector::OptionSignedInt(v),
            OptionNumericVector::OptionUnsignedInt(v) => Vector::OptionUnsignedInt(v),
        }
    }
}
impl CategoricalVector {
    pub(crate) fn to_vector(self) -> Vector {
        match self {
            CategoricalVector::Bool(v) => Vector::Bool(v),
            CategoricalVector::String(v) => Vector::String(v),
            CategoricalVector::SignedInt(v) => Vector::SignedInt(v),
            CategoricalVector::UnsignedInt(v) => Vector::UnsignedInt(v),
            CategoricalVector::FiniteFloat(v) => Vector::FiniteFloat(v),
        }
    }
}



// build higher-tier types from atomic type
macro_rules! impl_from {
    ($atomic_type:ty, $scalar_type:ty, $vector_type:ty) => {
        impl From<$atomic_type> for Scalar {
            fn from(x: $atomic_type) -> Self {
                Scalar::from(<$scalar_type>::from(x))
            }
        }
        impl From<Vec<$atomic_type>> for Vector {
            fn from(x: Vec<$atomic_type>) -> Self {
                Vector::from(<$vector_type>::from(x))
            }
        }
        impl From<&$atomic_type> for Scalar {
            fn from(x: &$atomic_type) -> Self {
                Scalar::from(<$scalar_type>::from(x.clone()))
            }
        }
        impl From<&Vec<$atomic_type>> for Vector {
            fn from(x: &Vec<$atomic_type>) -> Self {
                Vector::from(<$vector_type>::from(x.clone()))
            }
        }
    }
}
impl_from!(f64, FloatScalar, FloatVector);
impl_from!(f32, FloatScalar, FloatVector);
impl_from!(R64, FiniteFloatScalar, FiniteFloatVector);
impl_from!(R32, FiniteFloatScalar, FiniteFloatVector);
impl_from!(i8, SignedIntScalar, SignedIntVector);
impl_from!(i16, SignedIntScalar, SignedIntVector);
impl_from!(i32, SignedIntScalar, SignedIntVector);
impl_from!(i64, SignedIntScalar, SignedIntVector);
impl_from!(i128, SignedIntScalar, SignedIntVector);
impl_from!(u8, UnsignedIntScalar, UnsignedIntVector);
impl_from!(u16, UnsignedIntScalar, UnsignedIntVector);
impl_from!(u32, UnsignedIntScalar, UnsignedIntVector);
impl_from!(u64, UnsignedIntScalar, UnsignedIntVector);
impl_from!(u128, UnsignedIntScalar, UnsignedIntVector);
impl_from!(Option<i8>, OptionSignedIntScalar, OptionSignedIntVector);
impl_from!(Option<i16>, OptionSignedIntScalar, OptionSignedIntVector);
impl_from!(Option<i32>, OptionSignedIntScalar, OptionSignedIntVector);
impl_from!(Option<i64>, OptionSignedIntScalar, OptionSignedIntVector);
impl_from!(Option<i128>, OptionSignedIntScalar, OptionSignedIntVector);
impl_from!(Option<u8>, OptionUnsignedIntScalar, OptionUnsignedIntVector);
impl_from!(Option<u16>, OptionUnsignedIntScalar, OptionUnsignedIntVector);
impl_from!(Option<u32>, OptionUnsignedIntScalar, OptionUnsignedIntVector);
impl_from!(Option<u64>, OptionUnsignedIntScalar, OptionUnsignedIntVector);
impl_from!(Option<u128>, OptionUnsignedIntScalar, OptionUnsignedIntVector);
macro_rules! impl_numeric_from {
    ($atomic_type:ty, $scalar_type:ty, $vector_type:ty) => {
        impl From<$atomic_type> for NumericScalar {
            fn from(x: $atomic_type) -> Self {
                NumericScalar::from(<$scalar_type>::from(x))
            }
        }
        impl From<Vec<$atomic_type>> for NumericVector {
            fn from(x: Vec<$atomic_type>) -> Self {
                NumericVector::from(<$vector_type>::from(x))
            }
        }
        impl From<&$atomic_type> for NumericScalar {
            fn from(x: &$atomic_type) -> Self {
                NumericScalar::from(<$scalar_type>::from(x.clone()))
            }
        }
        impl From<&Vec<$atomic_type>> for NumericVector {
            fn from(x: &Vec<$atomic_type>) -> Self {
                NumericVector::from(<$vector_type>::from(x.clone()))
            }
        }
    }
}
impl_numeric_from!(R64, FiniteFloatScalar, FiniteFloatVector);
impl_numeric_from!(R32, FiniteFloatScalar, FiniteFloatVector);
impl_numeric_from!(i8, SignedIntScalar, SignedIntVector);
impl_numeric_from!(i16, SignedIntScalar, SignedIntVector);
impl_numeric_from!(i32, SignedIntScalar, SignedIntVector);
impl_numeric_from!(i64, SignedIntScalar, SignedIntVector);
impl_numeric_from!(i128, SignedIntScalar, SignedIntVector);
impl_numeric_from!(u8, UnsignedIntScalar, UnsignedIntVector);
impl_numeric_from!(u16, UnsignedIntScalar, UnsignedIntVector);
impl_numeric_from!(u32, UnsignedIntScalar, UnsignedIntVector);
impl_numeric_from!(u64, UnsignedIntScalar, UnsignedIntVector);
impl_numeric_from!(u128, UnsignedIntScalar, UnsignedIntVector);
macro_rules! impl_categorical_from {
    ($atomic_type:ty, $scalar_type:ty, $vector_type:ty) => {
        impl From<$atomic_type> for CategoricalScalar {
            fn from(x: $atomic_type) -> Self {
                CategoricalScalar::from(<$scalar_type>::from(x))
            }
        }
        impl From<Vec<$atomic_type>> for CategoricalVector {
            fn from(x: Vec<$atomic_type>) -> Self {
                CategoricalVector::from(<$vector_type>::from(x))
            }
        }
        impl From<&$atomic_type> for CategoricalScalar {
            fn from(x: &$atomic_type) -> Self {
                CategoricalScalar::from(<$scalar_type>::from(x.clone()))
            }
        }
        impl From<&Vec<$atomic_type>> for CategoricalVector {
            fn from(x: &Vec<$atomic_type>) -> Self {
                CategoricalVector::from(<$vector_type>::from(x.clone()))
            }
        }
    }
}
impl_categorical_from!(R64, FiniteFloatScalar, FiniteFloatVector);
impl_categorical_from!(R32, FiniteFloatScalar, FiniteFloatVector);
impl_categorical_from!(i8, SignedIntScalar, SignedIntVector);
impl_categorical_from!(i16, SignedIntScalar, SignedIntVector);
impl_categorical_from!(i32, SignedIntScalar, SignedIntVector);
impl_categorical_from!(i64, SignedIntScalar, SignedIntVector);
impl_categorical_from!(i128, SignedIntScalar, SignedIntVector);
impl_categorical_from!(u8, UnsignedIntScalar, UnsignedIntVector);
impl_categorical_from!(u16, UnsignedIntScalar, UnsignedIntVector);
impl_categorical_from!(u32, UnsignedIntScalar, UnsignedIntVector);
impl_categorical_from!(u64, UnsignedIntScalar, UnsignedIntVector);
impl_categorical_from!(u128, UnsignedIntScalar, UnsignedIntVector);
