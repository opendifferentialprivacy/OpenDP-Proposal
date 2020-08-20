use crate::Error;
use derive_more::{From};

use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub enum Data {
    Pointer(i64),
    // Literal(Value)
}

macro_rules! impl_get_variant {
    ($target:ty, $name:ident, $variant:path, $result:ty) => {

        impl $target {
            pub fn $name(&self) -> Result<&$result, Error> {
                match self {
                    $variant(x) => Ok(x),
                    _ => Err(Error::Raw("unexpected variant")),
                }
            }
        }
    };
}

#[derive(Clone, Debug)]
pub enum Value {
    Scalar(Scalar),
    Vector(Vector)
}

// SCALARS
#[derive(From, PartialEq, PartialOrd, Clone, Debug)]
pub enum NumericScalar {
    Float(FloatScalar),
    Int(IntScalar),
    OptionInt(Option<IntScalar>)
}

impl NumericScalar {
    pub fn max(self, other: &NumericScalar) -> NumericScalar {
        match self.partial_cmp(other) {
            Some(Ordering::Equal) | None => self,
            Some(Ordering::Greater) => self,
            Some(Ordering::Less) => other.clone()
        }
    }

    pub fn min(self, other: &NumericScalar) -> NumericScalar {
        match self.partial_cmp(other) {
            Some(Ordering::Equal) | None => self,
            Some(Ordering::Greater) => other.clone(),
            Some(Ordering::Less) => self
        }
    }
}

#[derive(From, PartialEq, Eq, Clone, Debug)]
pub enum CategoricalScalar {
    Bool(bool),
    OptionBool(Option<bool>),
    String(String),
    OptionString(Option<String>),
    Int(IntScalar),
    OptionInt(Option<IntScalar>)
}

#[derive(From, PartialEq, PartialOrd, Eq, Clone, Debug)]
pub enum IntScalar {
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

#[derive(From, PartialEq, PartialOrd, Clone, Debug)]
pub enum FloatScalar {
    F32(f32),
    F64(f64),
}

#[derive(From, Clone, Debug)]
pub enum Scalar {
    Bool(bool),
    OptionBool(Option<bool>),
    String(String),
    OptionString(Option<String>),
    Int(IntScalar),
    OptionInt(Option<IntScalar>),
    Float(FloatScalar),
}


impl Scalar {
    pub(crate) fn numeric(self) -> Result<NumericScalar, Error> {
        Ok(match self {
            Scalar::Int(v) => NumericScalar::Int(v),
            Scalar::OptionInt(v) => NumericScalar::OptionInt(v),
            Scalar::Float(v) => NumericScalar::Float(v),
            _ => return Err(Error::Raw("invalid atomic type"))
        })
    }
    pub(crate) fn categorical(self) -> Result<CategoricalScalar, Error> {
        Ok(match self {
            Scalar::Bool(v) => CategoricalScalar::Bool(v),
            Scalar::OptionBool(v) => CategoricalScalar::OptionBool(v),
            Scalar::String(v) => CategoricalScalar::String(v),
            Scalar::OptionString(v) => CategoricalScalar::OptionString(v),
            Scalar::Int(v) => CategoricalScalar::Int(v),
            Scalar::OptionInt(v) => CategoricalScalar::OptionInt(v),
            _ => return Err(Error::Raw("invalid atomic type"))
        })
    }
}

// VECTORS
#[derive(From, PartialEq, PartialOrd, Clone, Debug)]
pub enum NumericVector {
    Float(FloatVector),
    Int(IntVector),
    OptionInt(OptionIntVector)
}

#[derive(From, PartialEq, Eq, Clone, Debug)]
pub enum CategoricalVector {
    Bool(Vec<bool>),
    OptionBool(Vec<Option<bool>>),
    String(Vec<String>),
    OptionString(Vec<Option<String>>),
    Int(IntVector),
    OptionInt(OptionIntVector)
}

#[derive(From, PartialEq, PartialOrd, Eq, Clone, Debug)]
pub enum IntVector {
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
}

#[derive(From, PartialEq, PartialOrd, Eq, Clone, Debug)]
pub enum OptionIntVector {
    I128(Vec<Option<i128>>),
    I64(Vec<Option<i64>>),
    I32(Vec<Option<i32>>),
    I16(Vec<Option<i16>>),
    I8(Vec<Option<i8>>),
    U128(Vec<Option<u128>>),
    U64(Vec<Option<u64>>),
    U32(Vec<Option<u32>>),
    U16(Vec<Option<u16>>),
    U8(Vec<Option<u8>>),
}

#[derive(From, PartialEq, PartialOrd, Clone, Debug)]
pub enum FloatVector {
    F32(Vec<f32>),
    F64(Vec<f64>),
}

#[derive(From, Clone, Debug)]
pub enum Vector {
    Bool(Vec<bool>),
    OptionBool(Vec<Option<bool>>),
    String(Vec<String>),
    OptionString(Vec<Option<String>>),
    Int(IntVector),
    OptionInt(OptionIntVector),
    Float(FloatVector),
}

impl Vector {
    fn numeric(self) -> Result<NumericVector, Error> {
        Ok(match self {
            Vector::Int(v) => NumericVector::Int(v),
            Vector::OptionInt(v) => NumericVector::OptionInt(v),
            Vector::Float(v) => NumericVector::Float(v),
            _ => return Err(Error::Raw("invalid atomic type"))
        })
    }
    fn categorical(self) -> Result<CategoricalVector, Error> {
        Ok(match self {
            Vector::Bool(v) => CategoricalVector::Bool(v),
            Vector::OptionBool(v) => CategoricalVector::OptionBool(v),
            Vector::String(v) => CategoricalVector::String(v),
            Vector::OptionString(v) => CategoricalVector::OptionString(v),
            Vector::Int(v) => CategoricalVector::Int(v),
            Vector::OptionInt(v) => CategoricalVector::OptionInt(v),
            _ => return Err(Error::Raw("invalid atomic type"))
        })
    }
}


#[derive(From, PartialEq, Clone, Debug)]
pub enum Domain {
    Scalar(ScalarDomain),
    Vector(VectorDomain<i64>)
}

impl_get_variant!(Domain, scalar, Domain::Scalar, ScalarDomain);
impl_get_variant!(Domain, vector, Domain::Vector, VectorDomain<i64>);

#[derive(PartialEq, Clone, Debug)]
pub struct VectorDomain<TLength: PartialEq + Clone> {
    pub atomic_type: Box<Domain>,
    pub is_nonempty: bool,
    pub length: Option<TLength>
}

#[derive(From, PartialEq, Clone, Debug)]
pub enum ScalarDomain {
    Numeric(NumericDomain),
    Categorical(CategoricalDomain)
}
impl_get_variant!(ScalarDomain, numeric, ScalarDomain::Numeric, NumericDomain);
impl_get_variant!(ScalarDomain, categorical, ScalarDomain::Categorical, CategoricalDomain);

#[derive(PartialEq, Clone, Debug)]
pub struct NumericDomain {
    pub(crate) lower: Option<NumericScalar>,
    pub(crate) upper: Option<NumericScalar>
}

impl NumericDomain {
    pub fn new(lower: Option<Scalar>, upper: Option<Scalar>) -> Result<NumericDomain, Error> {
        let lower = lower.map(|l| l.numeric()).transpose()?;
        let upper = upper.map(|u| u.numeric()).transpose()?;

        if let (Some(l), Some(u)) = (&lower, &upper) {
            if !l.eq_type(&u) {
                return Err(Error::DomainMismatch)
            }
        }

        // check that lower/upper are not null or optional
        macro_rules! is_finite {
            ($bound:expr) => {
                match $bound {
                    Some(NumericScalar::Float(FloatScalar::F64(v))) => if !v.is_finite() {return Err(Error::DomainMismatch)},
                    Some(NumericScalar::Float(FloatScalar::F32(v))) => if !v.is_finite() {return Err(Error::DomainMismatch)},
                    _ => ()
                }
            }
        }
        is_finite!(lower);
        is_finite!(upper);

        Ok(NumericDomain {lower, upper})
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct CategoricalDomain(CategoricalVector);
impl CategoricalDomain {
    fn new(categories: Vector) -> Result<CategoricalDomain, Error> {
        Ok(CategoricalDomain(categories.categorical()?))
    }
}

macro_rules! impl_numeric_eqtype {
    ($numericType:ty) => {
        impl EqType for $numericType {
            fn eq_type(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::Float(l), Self::Float(r)) => l.eq_type(r),
                    (Self::Int(l), Self::Int(r)) => l.eq_type(r),
                    (Self::OptionInt(l), Self::OptionInt(r)) => l.eq_type(r),
                    _ => false
                }
            }
        }
    }
}
macro_rules! impl_categorical_eqtype {
    ($categoricalType:ty) => {
        impl EqType for $categoricalType {
            fn eq_type(&self, other: &Self) -> bool {
                if std::mem::discriminant(self) != std::mem::discriminant(other) {
                    return false;
                }
                match (self, other) {
                    (Self::Int(l), Self::Int(r)) => l.eq_type(r),
                    _ => true
                }
            }
        }
    }
}


trait EqType: Sized {
    fn eq_type(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl_numeric_eqtype!(NumericScalar);
impl_categorical_eqtype!(CategoricalScalar);
impl EqType for IntScalar {}
impl EqType for FloatScalar {}
impl EqType for Option<IntScalar> {
    fn eq_type(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(l), Some(r)) => l.eq_type(r),
            (None, None) => true,
            _ => false
        }
    }
}

impl_numeric_eqtype!(NumericVector);
impl_categorical_eqtype!(CategoricalVector);
impl EqType for IntVector {}
impl EqType for FloatVector {}
impl EqType for OptionIntVector {}

// serialization necessary for composing measurements constructed across multiple processes

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
impl_scalar_from!(i8, IntScalar);
impl_scalar_from!(i16, IntScalar);
impl_scalar_from!(i32, IntScalar);
impl_scalar_from!(i64, IntScalar);
impl_scalar_from!(i128, IntScalar);
impl_scalar_from!(u8, IntScalar);
impl_scalar_from!(u16, IntScalar);
impl_scalar_from!(u32, IntScalar);
impl_scalar_from!(u64, IntScalar);
impl_scalar_from!(u128, IntScalar);
