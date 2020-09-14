use std::cmp::Ordering;

use indexmap::map::IndexMap;


use opendp_derive::{AutoFrom, AutoGet};

use crate::base::value::*;
use crate::Error;
use crate::base::functions::deduplicate;


#[derive(PartialEq, Clone, Debug, AutoFrom, AutoGet)]
pub enum Domain {
    Scalar(ScalarDomain),
    Vector(VectorDomain),
    Dataframe(DataframeDomain),
}

#[derive(PartialEq, Clone, Debug)]
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

#[derive(PartialEq, Clone, Debug, AutoFrom, AutoGet)]
pub enum Nature {
    Numeric(Interval),
    Categorical(Categories),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Categories(CategoricalVector);

impl Categories {
    pub fn new(values: CategoricalVector) -> Categories {
        Categories(apply_categorical_vector!(deduplicate, values).unwrap())
    }
    pub fn get(self) -> CategoricalVector { self.0 }
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


impl DataframeDomain {
    pub fn assert_non_null(&self) -> Result<(), Error> {
        for atomic_type in self.columns.values() {
            atomic_type.assert_non_null()?
        }
        Ok(())
    }
}

impl ScalarDomain {
    pub fn assert_non_null(&self) -> Result<(), Error> {
        if self.may_have_nullity {
            Err(Error::PotentialNullity)
        } else { Ok(()) }
    }
}
