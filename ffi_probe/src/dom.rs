use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Bound;

use crate::core::Domain;
use crate::data::{Data, Form};

/// A Domain that contains all members of the carrier type.
pub struct AllDomain<T> {
    _marker: PhantomData<T>,
}
impl<T> AllDomain<T> {
    pub fn new() -> Self {
        AllDomain { _marker: PhantomData }
    }
}
impl<T> Clone for AllDomain<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<T> Domain for AllDomain<T> {
    type Carrier = T;
    fn check_compatible(&self, _other: &Self) -> bool { true }
    fn check_valid(&self, _val: &Self::Carrier) -> bool { true }
}


/// A Domain that unwraps a Data wrapper.
pub struct DataDomain<D: Domain> {
    pub form_domain: D,
}
impl<D: Domain> DataDomain<D> {
    pub fn new(form_domain: D) -> Self {
        DataDomain { form_domain }
    }
}
impl<D: Domain> Clone for DataDomain<D> {
    fn clone(&self) -> Self {
        DataDomain::new(self.form_domain.clone())
    }
}
impl<D: Domain> Domain for DataDomain<D> where
    D::Carrier: 'static + Form {
    type Carrier = Data;
    fn check_compatible(&self, other: &Self) -> bool {
        self.form_domain.check_compatible(&other.form_domain)
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        let val = val.as_form();
        self.form_domain.check_valid(val)
    }
}


/// A Domain that contains all the values in an interval.
#[derive(Clone, PartialEq)]
pub struct IntervalDomain<T> {
    pub lower: Bound<T>,
    pub upper: Bound<T>,
}
impl<T> IntervalDomain<T> {
    pub fn new(lower: Bound<T>, upper: Bound<T>) -> Self {
        IntervalDomain { lower, upper }
    }
}
impl<T: Clone + PartialOrd> Domain for IntervalDomain<T> {
    type Carrier = T;
    fn check_compatible(&self, other: &Self) -> bool {
        self == other
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        let lower_ok = match &self.lower {
            Bound::Included(bound) => { val >= bound }
            Bound::Excluded(bound) => { val > bound }
            Bound::Unbounded => { true }
        };
        lower_ok && match &self.upper {
            Bound::Included(bound) => { val <= bound }
            Bound::Excluded(bound) => { val < bound }
            Bound::Unbounded => { true }
        }
    }
}


/// A Domain that contains pairs of values.
pub struct PairDomain<D0: Domain, D1: Domain>(pub D0, pub D1);
impl<D0: Domain, D1: Domain> PairDomain<D0, D1> {
    pub fn new(element_domain0: D0, element_domain1: D1) -> Self {
        PairDomain(element_domain0, element_domain1)
    }
}
impl<D0: Domain, D1: Domain> Clone for PairDomain<D0, D1> {
    fn clone(&self) -> Self {
        PairDomain(self.0.clone(), self.1.clone())
    }
}
impl<D0: Domain, D1: Domain> Domain for PairDomain<D0, D1> {
    type Carrier = (D0::Carrier, D1::Carrier);
    fn check_compatible(&self, other: &Self) -> bool {
        self.0.check_compatible(&other.0) && self.1.check_compatible(&other.1)
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        self.0.check_valid(&val.0) && self.1.check_valid(&val.1)
    }
}


/// A Domain that contains maps of (homogeneous) values.
#[derive(Clone)]
pub struct MapDomain<D: Domain> {
    pub element_domain: D
}
impl<D: Domain> MapDomain<D> {
    pub fn new(element_domain: D) -> Self {
        MapDomain { element_domain }
    }
}
impl<T> MapDomain<AllDomain<T>> {
    pub fn new_all() -> Self {
        Self::new(AllDomain::<T>::new())
    }
}
impl<D: Domain> Domain for MapDomain<D> {
    type Carrier = HashMap<String, D::Carrier>;
    fn check_compatible(&self, other: &Self) -> bool {
        self.element_domain.check_compatible(&other.element_domain)
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|e| self.element_domain.check_valid(e.1))
    }
}


/// A Domain that contains vectors of (homogeneous) values.
#[derive(Clone)]
pub struct VectorDomain<D: Domain> {
    pub element_domain: D,
}
impl<D: Domain> VectorDomain<D> {
    pub fn new(element_domain: D) -> Self {
        VectorDomain { element_domain }
    }
}
impl<T> VectorDomain<AllDomain<T>> {
    pub fn new_all() -> Self {
        Self::new(AllDomain::<T>::new())
    }
}
impl<D: Domain> Domain for VectorDomain<D> {
    type Carrier = Vec<D::Carrier>;
    fn check_compatible(&self, other: &Self) -> bool {
        self.element_domain.check_compatible(&other.element_domain)
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|e| self.element_domain.check_valid(e))
    }
}
