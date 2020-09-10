use indexmap::map::IndexMap;
use crate::base::value::{CategoricalVector, NumericScalar};
use std::cmp::Ordering;
use crate::Error;

#[derive(derive_more::From, PartialEq, Clone, Debug)]
pub enum Domain {
    Scalar(ScalarDomain),
    Vector(VectorDomain),
    Dataframe(DataframeDomain),
}

#[derive(derive_more::From, PartialEq, Clone, Debug)]
pub struct ScalarDomain {
    pub may_have_nullity: bool,
    pub nature: Nature,
}

#[derive(PartialEq, Clone, Debug)]
pub struct VectorDomain {
    pub atomic_type: Box<Domain>,
    pub is_nonempty: bool,
    pub length: Option<usize>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct DataframeDomain {
    pub columns: IndexMap<String, Domain>,
    pub is_nonempty: bool,
    pub length: Option<usize>,
}

#[derive(derive_more::From, PartialEq, Clone, Debug)]
pub enum Nature {
    Numeric(Interval),
    Categorical(Categories),
}

pub struct Categories(CategoricalVector);
impl Categories {
    pub fn new(values: CategoricalVector) -> Categories {
        values.apply()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Interval(Option<NumericScalar>, Option<NumericScalar>);

// IMPLEMENTATIONS
impl Interval {
    pub fn new(lower: Option<NumericScalar>, upper: Option<NumericScalar>) -> Result<Interval, Error> {
        if let (Some(l), Some(u)) = (&lower, &upper) {
            match l.partial_cmp(u) {
                None => return Err(crate::Error::AtomicMismatch),
                Some(Ordering::Greater) => return Err(crate::Error::InvalidDomain),
                _ => ()
            }
        }
        Ok(Interval(lower, upper))
    }

    pub fn bounds(self) -> (Option<NumericScalar>, Option<NumericScalar>) {
        (self.0, self.1)
    }
}

impl Domain {
    // TODO: should we have domain constructors at this fine granularity?
    pub fn numeric_scalar(
        lower: Option<NumericScalar>, upper: Option<NumericScalar>, may_have_nullity: bool,
    ) -> Result<Self, Error> {
        Ok(Domain::Scalar(ScalarDomain {
            may_have_nullity,
            nature: Nature::Numeric(Interval::new(lower, upper)?),
        }))
    }

    pub fn assert_non_null(&self) -> Result<(), Error> {
        Ok(match self {
            Domain::Scalar(domain) => domain.assert_non_null()?,
            Domain::Vector(domain) => domain.atomic_type.assert_non_null()?,
            Domain::Dataframe(domain) => domain.assert_non_null()?,
        })
    }
}


macro_rules! impl_get_variant {
    ($target:ty, $name:ident, $variant:path, $result:ty) => {

        impl $target {
            pub fn $name(&self) -> Result<&$result, Error> {
                match self {
                    $variant(x) => Ok(x),
                    _ => Err(Error::AtomicMismatch),
                }
            }
        }
    }
}

impl_get_variant!(Domain, scalar, Domain::Scalar, ScalarDomain);
impl_get_variant!(Domain, vector, Domain::Vector, VectorDomain);


impl DataframeDomain {
    pub fn assert_non_null(&self) -> Result<(), Error> {
        for atomic_type in self.columns.values() {
            atomic_type.assert_non_null()?
        }
        Ok(())
    }
}

impl_get_variant!(Nature, numeric, Nature::Numeric, Interval);
impl_get_variant!(Nature, categorical, Nature::Categorical, CategoricalVector);


impl ScalarDomain {
    pub fn assert_non_null(&self) -> Result<(), Error> {
        if self.may_have_nullity {
            Err(Error::PotentialNullity)
        } else { Ok(()) }
    }
}
