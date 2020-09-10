use std::cmp::Ordering;
use std::fmt::Debug;

use indexmap::map::IndexMap;
use noisy_float::types::{R32, R64};

use crate::Error;
use std::ops::Mul;

#[derive(Clone, Debug)]
pub enum Value {
    Scalar(Scalar),
    Vector(Vector),
    Dataframe(Dataframe),
}

#[derive(Clone, Debug)]
pub struct Dataframe(pub IndexMap<CategoricalScalar, Value>);


// ~~~~ SCALAR ~~~~
// TYPES

#[derive(Clone, Debug, derive_more::From)]
pub enum Scalar {
    Bool(bool),
    OptionBool(Option<bool>),

    String(String),
    OptionString(Option<String>),

    SignedInt(SignedIntScalar),
    OptionSignedInt(OptionSignedIntScalar),

    UnsignedInt(UnsignedIntScalar),
    OptionUnsignedInt(OptionUnsignedIntScalar),

    FiniteFloat(FiniteFloatScalar),
    Float(FloatScalar),
}

#[derive(PartialEq, Eq, Clone, Debug, derive_more::From)]
pub enum UnsignedIntScalar {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

#[derive(PartialEq, Eq, Clone, Debug, derive_more::From)]
pub enum SignedIntScalar {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

#[derive(PartialEq, Eq, Clone, Debug, derive_more::From)]
pub enum OptionSignedIntScalar {
    I8(Option<i8>),
    I16(Option<i16>),
    I32(Option<i32>),
    I64(Option<i64>),
    I128(Option<i128>),
}

#[derive(PartialEq, Eq, Clone, Debug, derive_more::From)]
pub enum OptionUnsignedIntScalar {
    U8(Option<u8>),
    U16(Option<u16>),
    U32(Option<u32>),
    U64(Option<u64>),
    U128(Option<u128>),
}

#[derive(derive_more::From, PartialEq, Eq, Ord, Clone, Debug)]
pub enum FiniteFloatScalar {
    F32(R32),
    F64(R64),
}

#[derive(derive_more::From, PartialEq, Clone, Debug)]
pub enum FloatScalar {
    F32(f32),
    F64(f64),
}


// SUBSET TYPES
#[derive(PartialEq, Eq, Clone, Debug, derive_more::From)]
pub enum NumericScalar {
    FiniteFloat(FiniteFloatScalar),
    SignedInt(SignedIntScalar),
    UnsignedInt(UnsignedIntScalar)
}

#[derive(PartialEq, Clone, Debug, derive_more::From)]
pub enum OptionNumericScalar {
    Float(FloatScalar),
    OptionSignedInt(OptionSignedIntScalar),
    OptionUnsignedInt(OptionUnsignedIntScalar),
}

#[derive(derive_more::From, PartialEq, Eq, Clone, Debug)]
pub enum CategoricalScalar {
    Bool(bool),
    String(String),
    SignedInt(SignedIntScalar),
    UnsignedInt(UnsignedIntScalar),
    FiniteFloat(FiniteFloatScalar)
}

// TRAIT IMPLEMENTATIONS
// Trait : PartialOrd
impl PartialOrd for SignedIntScalar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use SignedIntScalar::*;
        match (self, other) {
            (I8(x), I8(y)) => x.partial_cmp(y),
            (I16(x), I16(y)) => x.partial_cmp(y),
            (I32(x), I32(y)) => x.partial_cmp(y),
            (I64(x), I64(y)) => x.partial_cmp(y),
            (I128(x), I128(y)) => x.partial_cmp(y),
            _ => None
        }
    }
}
impl PartialOrd for OptionUnsignedIntScalar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use OptionUnsignedIntScalar::*;
        match (self, other) {
            (U8(x), U8(y)) => x.partial_cmp(y),
            (U16(x), U16(y)) => x.partial_cmp(y),
            (U32(x), U32(y)) => x.partial_cmp(y),
            (U64(x), U64(y)) => x.partial_cmp(y),
            (U128(x), U128(y)) => x.partial_cmp(y),
            _ => None
        }
    }
}
impl PartialOrd for OptionSignedIntScalar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use OptionSignedIntScalar::*;
        match (self, other) {
            (I8(x), I8(y)) => x.partial_cmp(y),
            (I16(x), I16(y)) => x.partial_cmp(y),
            (I32(x), I32(y)) => x.partial_cmp(y),
            (I64(x), I64(y)) => x.partial_cmp(y),
            (I128(x), I128(y)) => x.partial_cmp(y),
            _ => None
        }
    }
}
impl PartialOrd for UnsignedIntScalar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use UnsignedIntScalar::*;
        match (self, other) {
            (U8(x), U8(y)) => x.partial_cmp(y),
            (U16(x), U16(y)) => x.partial_cmp(y),
            (U32(x), U32(y)) => x.partial_cmp(y),
            (U64(x), U64(y)) => x.partial_cmp(y),
            (U128(x), U128(y)) => x.partial_cmp(y),
            _ => None
        }
    }
}
impl PartialOrd for FiniteFloatScalar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use FiniteFloatScalar::*;
        match (self, other) {
            (F32(x), F32(y)) => x.partial_cmp(y),
            (F64(x), F64(y)) => x.partial_cmp(y),
            _ => None
        }
    }
}
impl PartialOrd for FloatScalar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use FloatScalar::*;
        match (self, other) {
            (F64(l), F64(r)) => l.partial_cmp(r),
            (F32(l), F32(r)) => l.partial_cmp(r),
            _ => None
        }
    }
}
impl PartialOrd for NumericScalar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use NumericScalar::*;
        match (self, other) {
            (FiniteFloat(x), FiniteFloat(y)) => x.partial_cmp(y),
            (SignedInt(x), SignedInt(y)) => x.partial_cmp(y),
            (UnsignedInt(x), UnsignedInt(y)) => x.partial_cmp(y),
            _ => None
        }
    }
}
impl PartialOrd for OptionNumericScalar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use OptionNumericScalar::*;
        match (self, other) {
            (Float(x), Float(y)) => x.partial_cmp(y),
            (OptionSignedInt(x), OptionSignedInt(y)) => x.partial_cmp(y),
            (OptionUnsignedInt(x), OptionUnsignedInt(y)) => x.partial_cmp(y),
            _ => None
        }
    }
}


// IMPLEMENTATIONS
// specialize to subset type
impl Scalar {
    pub(crate) fn numeric(self) -> Result<NumericScalar, Error> {
        Ok(match self {
            Scalar::SignedInt(v) => NumericScalar::SignedInt(v),
            Scalar::UnsignedInt(v) => NumericScalar::UnsignedInt(v),
            Scalar::FiniteFloat(v) => NumericScalar::FiniteFloat(v),
            _ => return Err(Error::AtomicMismatch)
        })
    }
    pub(crate) fn categorical(self) -> Result<CategoricalScalar, Error> {
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
    pub(crate) fn scalar(self) -> Scalar {
        match self {
            NumericScalar::FiniteFloat(v) => Scalar::FiniteFloat(v),
            NumericScalar::SignedInt(v) => Scalar::SignedInt(v),
            NumericScalar::UnsignedInt(v) => Scalar::UnsignedInt(v),
        }
    }
}
impl OptionNumericScalar {
    pub(crate) fn scalar(self) -> Scalar {
        match self {
            OptionNumericScalar::Float(v) => Scalar::Float(v),
            OptionNumericScalar::OptionSignedInt(v) => Scalar::OptionSignedInt(v),
            OptionNumericScalar::OptionUnsignedInt(v) => Scalar::OptionUnsignedInt(v),
        }
    }
}
impl CategoricalScalar {
    pub(crate) fn scalar(self) -> Scalar {
        match self {
            CategoricalScalar::Bool(v) => Scalar::Bool(v),
            CategoricalScalar::String(v) => Scalar::String(v),
            CategoricalScalar::SignedInt(v) => Scalar::SignedInt(v),
            CategoricalScalar::UnsignedInt(v) => Scalar::UnsignedInt(v),
            CategoricalScalar::FiniteFloat(v) => Scalar::FiniteFloat(v),
        }
    }
}

// utility functions
// min and max
macro_rules! impl_minmax {
    ($target:ty) => {
        impl $target {
            pub fn max(&self, other: &$target) -> Result<$target, Error> {
                Ok(match self.partial_cmp(other) {
                    Some(Ordering::Equal) | Some(Ordering::Greater) => self,
                    Some(Ordering::Less) => other,
                    None => return Err(crate::Error::AtomicMismatch)
                }.clone())
            }

            pub fn min(&self, other: &$target) -> Result<$target, Error> {
                Ok(match self.partial_cmp(other) {
                    Some(Ordering::Equal) | Some(Ordering::Less) => self,
                    Some(Ordering::Greater) => other,
                    None => return Err(crate::Error::AtomicMismatch)
                }.clone())
            }
        }
    }
}
impl_minmax!(SignedIntScalar);
impl_minmax!(UnsignedIntScalar);
impl_minmax!(OptionSignedIntScalar);
impl_minmax!(OptionUnsignedIntScalar);
impl_minmax!(FiniteFloatScalar);
impl_minmax!(FloatScalar);
impl_minmax!(NumericScalar);
impl_minmax!(OptionNumericScalar);

// build from atomic type
macro_rules! impl_scalar_from {
    ($scalar_type:ty, $enum_type:ty) => {
        impl From<$scalar_type> for Scalar {
            fn from(x: $scalar_type) -> Self {
                Scalar::from(<$enum_type>::from(x))
            }
        }
    }
}
impl_scalar_from!(f64, FloatScalar);
impl_scalar_from!(f32, FloatScalar);
impl_scalar_from!(R64, FiniteFloatScalar);
impl_scalar_from!(R32, FiniteFloatScalar);
impl_scalar_from!(i8, SignedIntScalar);
impl_scalar_from!(i16, SignedIntScalar);
impl_scalar_from!(i32, SignedIntScalar);
impl_scalar_from!(i64, SignedIntScalar);
impl_scalar_from!(i128, SignedIntScalar);
impl_scalar_from!(u8, UnsignedIntScalar);
impl_scalar_from!(u16, UnsignedIntScalar);
impl_scalar_from!(u32, UnsignedIntScalar);
impl_scalar_from!(u64, UnsignedIntScalar);
impl_scalar_from!(u128, UnsignedIntScalar);
impl_scalar_from!(Option<i8>, OptionSignedIntScalar);
impl_scalar_from!(Option<i16>, OptionSignedIntScalar);
impl_scalar_from!(Option<i32>, OptionSignedIntScalar);
impl_scalar_from!(Option<i64>, OptionSignedIntScalar);
impl_scalar_from!(Option<i128>, OptionSignedIntScalar);
impl_scalar_from!(Option<u8>, OptionUnsignedIntScalar);
impl_scalar_from!(Option<u16>, OptionUnsignedIntScalar);
impl_scalar_from!(Option<u32>, OptionUnsignedIntScalar);
impl_scalar_from!(Option<u64>, OptionUnsignedIntScalar);
impl_scalar_from!(Option<u128>, OptionUnsignedIntScalar);


// ~~~~ VECTOR ~~~~
// TYPES
#[derive(Clone, Debug, derive_more::From)]
pub enum Vector {
    Bool(Vec<bool>),
    OptionBool(Vec<Option<bool>>),

    String(Vec<String>),
    OptionString(Vec<Option<String>>),

    SignedInt(SignedIntVector),
    OptionSignedInt(OptionSignedIntVector),

    UnsignedInt(UnsignedIntVector),
    OptionUnsignedInt(OptionUnsignedIntVector),

    FiniteFloat(FiniteFloatVector),
    Float(FloatVector),
}

#[derive(PartialEq, Eq, Clone, Debug, derive_more::From)]
pub enum SignedIntVector {
    I8(Vec<i8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    I128(Vec<i128>),
}

#[derive(PartialEq, Eq, Clone, Debug, derive_more::From)]
pub enum UnsignedIntVector {
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    U64(Vec<u64>),
    U128(Vec<u128>),
}

#[derive(PartialEq, Eq, Clone, Debug, derive_more::From)]
pub enum OptionSignedIntVector {
    I8(Vec<Option<i8>>),
    I16(Vec<Option<i16>>),
    I32(Vec<Option<i32>>),
    I64(Vec<Option<i64>>),
    I128(Vec<Option<i128>>),
}

#[derive(PartialEq, Eq, Clone, Debug, derive_more::From)]
pub enum OptionUnsignedIntVector {
    U8(Vec<Option<u8>>),
    U16(Vec<Option<u16>>),
    U32(Vec<Option<u32>>),
    U64(Vec<Option<u64>>),
    U128(Vec<Option<u128>>),
}

#[derive(derive_more::From, PartialEq, Eq, Clone, Debug)]
pub enum FiniteFloatVector {
    F32(Vec<R32>),
    F64(Vec<R64>),
}

#[derive(derive_more::From, PartialEq, Clone, Debug)]
pub enum FloatVector {
    F32(Vec<f32>),
    F64(Vec<f64>),
}


// SUBSET TYPES
#[derive(PartialEq, Clone, Debug, derive_more::From)]
pub enum NumericVector {
    FiniteFloat(FiniteFloatVector),
    SignedInt(SignedIntVector),
    UnsignedInt(UnsignedIntVector),
}

#[derive(PartialEq, Clone, Debug, derive_more::From)]
pub enum OptionNumericVector {
    Float(FloatVector),
    OptionSignedInt(OptionSignedIntVector),
    OptionUnsignedInt(OptionUnsignedIntVector),
}

#[derive(derive_more::From, PartialEq, Eq, Clone, Debug)]
pub enum CategoricalVector {
    Bool(Vec<bool>),
    String(Vec<String>),
    SignedInt(SignedIntVector),
    UnsignedInt(UnsignedIntVector),
    FiniteFloat(FiniteFloatVector)
}

// IMPLEMENTATIONS
// specialize to subset type
impl Vector {
    pub fn numeric(self) -> Result<NumericVector, Error> {
        Ok(match self {
            Vector::SignedInt(v) => NumericVector::SignedInt(v),
            Vector::UnsignedInt(v) => NumericVector::UnsignedInt(v),
            Vector::FiniteFloat(v) => NumericVector::FiniteFloat(v),
            _ => return Err(Error::AtomicMismatch)
        })
    }
    pub fn categorical(self) -> Result<CategoricalVector, Error> {
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
    pub(crate) fn vector(self) -> Vector {
        match self {
            NumericVector::FiniteFloat(v) => Vector::FiniteFloat(v),
            NumericVector::SignedInt(v) => Vector::SignedInt(v),
            NumericVector::UnsignedInt(v) => Vector::UnsignedInt(v),
        }
    }
}
impl OptionNumericVector {
    pub(crate) fn vector(self) -> Vector {
        match self {
            OptionNumericVector::Float(v) => Vector::Float(v),
            OptionNumericVector::OptionSignedInt(v) => Vector::OptionSignedInt(v),
            OptionNumericVector::OptionUnsignedInt(v) => Vector::OptionUnsignedInt(v),
        }
    }
}
impl CategoricalVector {
    pub(crate) fn vector(self) -> Vector {
        match self {
            CategoricalVector::Bool(v) => Vector::Bool(v),
            CategoricalVector::String(v) => Vector::String(v),
            CategoricalVector::SignedInt(v) => Vector::SignedInt(v),
            CategoricalVector::UnsignedInt(v) => Vector::UnsignedInt(v),
            CategoricalVector::FiniteFloat(v) => Vector::FiniteFloat(v),
        }
    }
}

// apply function to variant
impl SignedIntVector {
    pub fn apply<T>(
        self, fun: Box<dyn Fn(Vec<T>) -> Result<Vec<T>, Error>>
    ) -> Result<Self, Error>
        where T: Ord + PartialOrd + PartialEq + Eq + Clone + Debug {
        Ok(match self {
            SignedIntVector::I8(v) => SignedIntVector::I8(fun(v)?),
            SignedIntVector::I16(v) => SignedIntVector::I16(fun(v)?),
            SignedIntVector::I32(v) => SignedIntVector::I32(fun(v)?),
            SignedIntVector::I64(v) => SignedIntVector::I64(fun(v)?),
            SignedIntVector::I128(v) => SignedIntVector::I128(fun(v)?),
        })
    }
}

impl UnsignedIntVector {
    pub fn apply<T>(
        self, fun: impl Fn(Vec<T>) -> Result<Vec<T>, Error>
    ) -> Result<Self, Error>
        where T: Ord + PartialOrd + PartialEq + Eq + Clone + Debug {
        Ok(match self {
            UnsignedIntVector::U8(v) => UnsignedIntVector::U8(fun(v)?),
            UnsignedIntVector::U16(v) => UnsignedIntVector::U16(fun(v)?),
            UnsignedIntVector::U32(v) => UnsignedIntVector::U32(fun(v)?),
            UnsignedIntVector::U64(v) => UnsignedIntVector::U64(fun(v)?),
            UnsignedIntVector::U128(v) => UnsignedIntVector::U128(fun(v)?),
        })
    }
}

impl OptionSignedIntVector {
    pub fn apply<T>(
        self, fun: impl Fn(Vec<T>) -> Result<Vec<T>, Error>
    ) -> Result<Self, Error>
        where T: Ord + PartialOrd + PartialEq + Eq + Clone + Debug {
        Ok(match self {
            OptionSignedIntVector::I8(v) => OptionSignedIntVector::I8(fun(v)?),
            OptionSignedIntVector::I16(v) => OptionSignedIntVector::I16(fun(v)?),
            OptionSignedIntVector::I32(v) => OptionSignedIntVector::I32(fun(v)?),
            OptionSignedIntVector::I64(v) => OptionSignedIntVector::I64(fun(v)?),
            OptionSignedIntVector::I128(v) =>  OptionSignedIntVector::I128(fun(v)?),
        })
    }
}

impl OptionUnsignedIntVector {
    pub fn apply<T>(
        self, fun: impl Fn(Vec<T>) -> Result<Vec<T>, Error>
    ) -> Result<Self, Error>
        where T: Ord + PartialOrd + PartialEq + Eq + Clone + Debug {
        Ok(match self {
            OptionUnsignedIntVector::U8(v) => OptionUnsignedIntVector::U8(fun(v)?),
            OptionUnsignedIntVector::U16(v) => OptionUnsignedIntVector::U16(fun(v)?),
            OptionUnsignedIntVector::U32(v) => OptionUnsignedIntVector::U32(fun(v)?),
            OptionUnsignedIntVector::U64(v) => OptionUnsignedIntVector::U64(fun(v)?),
            OptionUnsignedIntVector::U128(v) => OptionUnsignedIntVector::U128(fun(v)?),
        })
    }
}


impl FiniteFloatVector {
    pub fn apply<T>(
        self, fun: impl Fn(Vec<T>) -> Result<Vec<T>, Error>
    ) -> Result<Self, Error>
        where T: Ord + PartialOrd + PartialEq + Eq + Clone + Debug {
        Ok(match self {
            FiniteFloatVector::F64(v) => FiniteFloatVector::F64(fun(v)?),
            FiniteFloatVector::F32(v) => FiniteFloatVector::F32(fun(v)?),
        })
    }
}

impl FloatVector {
    pub fn apply<T>(
        self, fun: impl Fn(Vec<T>) -> Result<Vec<T>, Error>
    ) -> Result<Self, Error>
        where T: PartialOrd + PartialEq + Clone + Debug {
        Ok(match self {
            FloatVector::F64(v) => FloatVector::F64(fun(v)?),
            FloatVector::F32(v) => FloatVector::F32(fun(v)?),
        })
    }
}

impl NumericVector {
    pub fn apply<T>(
        self, fun: impl Fn(Vec<T>) -> Result<Vec<T>, Error>
    ) -> Result<Self, Error>
        where T: PartialOrd + PartialEq + Clone + Debug {
        Ok(match self {
            NumericVector::FiniteFloat(v) => NumericVector::FiniteFloat(v.apply(fun)?),
            NumericVector::SignedInt(v) => NumericVector::SignedInt(v.apply(fun)?),
            NumericVector::UnsignedInt(v) => NumericVector::UnsignedInt(v.apply(fun)?),
        })
    }
}

impl OptionNumericVector {
    pub fn apply<T>(
        self, fun: impl Fn(Vec<T>) -> Result<Vec<T>, Error>
    ) -> Result<Self, Error>
        where T: PartialOrd + PartialEq + Clone + Debug {
        Ok(match self {
            OptionNumericVector::Float(v) => OptionNumericVector::Float(v.apply(fun)?),
            OptionNumericVector::OptionSignedInt(v) => OptionNumericVector::OptionSignedInt(v.apply(fun)?),
            OptionNumericVector::OptionUnsignedInt(v) => OptionNumericVector::OptionUnsignedInt(v.apply(fun)?),
        })
    }
}

impl CategoricalVector {
    pub fn apply<T>(
        self, fun: impl Fn(Vec<T>) -> Result<Vec<T>, Error>
    ) -> Result<Self, Error>
    where T: PartialEq + Eq + Clone + Debug {
        Ok(match self {
            CategoricalVector::Bool(v) => CategoricalVector::Bool(fun(v)?),
            CategoricalVector::String(v) => CategoricalVector::String(fun(v)?),
            CategoricalVector::SignedInt(v) => CategoricalVector::SignedInt(v.apply(fun)?),
            CategoricalVector::UnsignedInt(v) => CategoricalVector::UnsignedInt(v.apply(fun)?),
            CategoricalVector::FiniteFloat(v) => CategoricalVector::FiniteFloat(v.apply(fun)?),
        })
    }
}
