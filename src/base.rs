use std::cmp::Ordering;

use indexmap::map::IndexMap;

use crate::Error;

#[derive(Clone, Debug)]
pub enum Data {
    Pointer(i64),
    Literal(Value),
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
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Scalar(Scalar),
    Vector(Vector)
}

// SCALARS
#[derive(
PartialEq, PartialOrd, Clone, Debug,
derive_more::From
)]
pub enum NumericScalar {
    Float(FloatScalar),
    Int(IntScalar),
    OptionInt(Option<IntScalar>)
}

impl NumericScalar {
    pub fn max(&self, other: &NumericScalar) -> Result<NumericScalar, crate::Error> {
        Ok(match self.partial_cmp(other) {
            Some(Ordering::Equal) => self,
            Some(Ordering::Greater) => self,
            Some(Ordering::Less) => other,
            None => return Err(crate::Error::AtomicMismatch)
        }.clone())
    }

    pub fn min(&self, other: &NumericScalar) -> Result<NumericScalar, crate::Error> {
        Ok(match self.partial_cmp(other) {
            Some(Ordering::Equal) => self,
            Some(Ordering::Greater) => other,
            Some(Ordering::Less) => self,
            None => return Err(crate::Error::AtomicMismatch)
        }.clone())
    }
}

#[derive(derive_more::From, PartialEq, Eq, Clone, Debug)]
pub enum CategoricalScalar {
    Bool(bool),
    OptionBool(Option<bool>),
    String(String),
    OptionString(Option<String>),
    Int(IntScalar),
    OptionInt(Option<IntScalar>)
}

#[derive(
PartialEq, PartialOrd, Eq, Clone, Debug,
derive_more::Add, derive_more::Sub, derive_more::From
)]
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

#[derive(
derive_more::From, PartialEq, PartialOrd, Clone, Debug,
derive_more::Add, derive_more::Sub
)]
pub enum FloatScalar {
    F32(f32),
    F64(f64),
}

#[derive(Clone, Debug, derive_more::From)]
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
#[derive(derive_more::From, PartialEq, PartialOrd, Clone, Debug)]
pub enum NumericVector {
    Float(FloatVector),
    Int(IntVector),
    OptionInt(OptionIntVector)
}

#[derive(derive_more::From, PartialEq, Eq, Clone, Debug)]
pub enum CategoricalVector {
    Bool(Vec<bool>),
    OptionBool(Vec<Option<bool>>),
    String(Vec<String>),
    OptionString(Vec<Option<String>>),
    Int(IntVector),
    OptionInt(OptionIntVector)
}

#[derive(derive_more::From, PartialEq, PartialOrd, Eq, Clone, Debug)]
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

#[derive(derive_more::From, PartialEq, PartialOrd, Eq, Clone, Debug)]
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

#[derive(derive_more::From, PartialEq, PartialOrd, Clone, Debug)]
pub enum FloatVector {
    F32(Vec<f32>),
    F64(Vec<f64>),
}

#[derive(derive_more::From, Clone, Debug)]
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


#[derive(derive_more::From, PartialEq, Clone, Debug)]
pub enum Domain {
    Scalar(ScalarDomain),
    Vector(VectorDomain<usize>),
    Matrix(MatrixDomain<usize>),
    Dataframe(DataframeDomain<usize>)
}

impl Domain {
    // TODO: should we have domain constructors at this fine granularity?
    pub fn numeric_scalar(
        lower: Option<NumericScalar>, upper: Option<NumericScalar>, may_have_nullity: bool,
    ) -> Result<Self, crate::Error> {
        Ok(Domain::Scalar(ScalarDomain {
            may_have_nullity,
            nature: Nature::Numeric(NumericDomain::new(lower, upper)?),
        }))
    }

    pub fn assert_non_null(&self) -> Result<(), crate::Error> {
        Ok(match self {
            Domain::Scalar(domain) => domain.assert_non_null()?,
            Domain::Vector(domain) => domain.atomic_type.assert_non_null()?,
            Domain::Dataframe(domain) => domain.assert_non_null()?,
            Domain::Matrix(domain) => domain.assert_non_null()?
        })
    }
}

impl_get_variant!(Domain, scalar, Domain::Scalar, ScalarDomain);
impl_get_variant!(Domain, vector, Domain::Vector, VectorDomain<usize>);

#[derive(PartialEq, Clone, Debug)]
pub struct VectorDomain<TLength: PartialEq + Clone> {
    pub atomic_type: Box<Domain>,
    pub is_nonempty: bool,
    pub length: Option<TLength>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct MatrixDomain<TLength: PartialEq + Clone> {
    pub atomic_type: Box<Domain>,
    pub is_nonempty: bool,
    pub length: Option<TLength>,
    pub columns: i64,
}

impl<TLength: Clone + PartialOrd> MatrixDomain<TLength> {
    pub fn assert_non_null(&self) -> Result<(), crate::Error> {
        self.atomic_type.assert_non_null()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct DataframeDomain<TLength: PartialEq + Clone> {
    pub columns: IndexMap<String, Domain>,
    pub is_nonempty: bool,
    pub length: Option<TLength>,
}

impl<TLength: Clone + PartialOrd> DataframeDomain<TLength> {
    pub fn assert_non_null(&self) -> Result<(), crate::Error> {
        for atomic_type in self.columns.values() {
            atomic_type.assert_non_null()?
        }
        Ok(())
    }
}

#[derive(derive_more::From, PartialEq, Clone, Debug)]
pub enum Nature {
    Numeric(NumericDomain),
    Categorical(CategoricalDomain),
}

impl_get_variant!(Nature, numeric, Nature::Numeric, NumericDomain);
impl_get_variant!(Nature, categorical, Nature::Categorical, CategoricalDomain);

#[derive(derive_more::From, PartialEq, Clone, Debug)]
pub struct ScalarDomain {
    pub may_have_nullity: bool,
    pub nature: Nature,
}

impl ScalarDomain {
    pub fn assert_non_null(&self) -> Result<(), crate::Error> {
        if self.may_have_nullity {
            Err(Error::PotentialNullity)
        } else { Ok(()) }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct NumericDomain {
    pub(crate) lower: Option<NumericScalar>,
    pub(crate) upper: Option<NumericScalar>,
}

impl NumericDomain {
    pub fn new(lower: Option<NumericScalar>, upper: Option<NumericScalar>) -> Result<NumericDomain, Error> {
        // let lower = lower.map(|l| l.numeric()).transpose()?;
        // let upper = upper.map(|u| u.numeric()).transpose()?;

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

// impl<T> Mul<T> for NumericScalar
//     where T: From<i64> {
//     type Output = NumericScalar;
//
//     fn mul(self, rhs: T) -> Self::Output {
//         let rhs = i64::from(rhs);
//         match self {
//             NumericScalar::Float(v) => NumericScalar::Float(v * rhs),
//             NumericScalar::Int(v) => NumericScalar::Int(v * rhs),
//             NumericScalar::OptionInt(v) => NumericScalar::OptionInt(v.map(|v| v * rhs)),
//         }
//     }
// }
//
// impl<T> Mul<T> for FloatScalar
//     where T: From<i64> {
//     type Output = FloatScalar;
//
//     fn mul(self, rhs: T) -> Self::Output {
//         let rhs = i64::from(rhs);
//         match self {
//             FloatScalar::F32(v) => FloatScalar::F32(v * rhs as f32),
//             FloatScalar::F64(v) => FloatScalar::F64(v * rhs as f64),
//         }
//     }
// }
//
// impl<T> Mul<T> for IntScalar
//     where T: From<i64> {
//     type Output = IntScalar;
//
//     fn mul(self, rhs: T) -> Self::Output {
//         let rhs = i64::from(rhs);
//         match self {
//             IntScalar::I8(v) => IntScalar::I8(v * rhs as i8),
//             IntScalar::I16(v) => IntScalar::I16(v * rhs as i16),
//             IntScalar::I32(v) => IntScalar::I32(v * rhs as i32),
//             IntScalar::I64(v) => IntScalar::I64(v * rhs as i64),
//             IntScalar::I128(v) => IntScalar::I128(v * rhs as i128),
//             IntScalar::U8(v) => IntScalar::U8(v * rhs as u8),
//             IntScalar::U16(v) => IntScalar::U16(v * rhs as u16),
//             IntScalar::U32(v) => IntScalar::U32(v * rhs as u32),
//             IntScalar::U64(v) => IntScalar::U64(v * rhs as u64),
//             IntScalar::U128(v) => IntScalar::U128(v * rhs as u128),
//         }
//     }
// }
//
// impl Mul<NumericScalar> for NumericScalar {
//     type Output = Result<NumericScalar, crate::Error>;
//
//     fn mul(self, rhs: NumericScalar) -> Self::Output {
//         unimplemented!()
//     }
// }
