use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Bound;

use crate::core::{Domain, DomainAlt};
use crate::data::{Data, Form, TraitObject};


/// A Domain that contains all members of the carrier type.
#[derive(PartialEq)]
pub struct AllDomain<T> {
    _marker: PhantomData<T>,
}
impl<T> AllDomain<T> {
    pub fn new() -> AllDomain<T> {
        AllDomain { _marker: PhantomData }
    }
}
impl<T> Clone for AllDomain<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<T: 'static> TraitObject for AllDomain<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static> Domain for AllDomain<T> {
    type Carrier = T;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>> {
        Box::new(Self::clone(self))
    }
    fn check_compatible(&self, _other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        true
    }
    fn check_valid(&self, _val: &Self::Carrier) -> bool { true }
}

pub struct AllDomainAlt<T> {
    _marker: PhantomData<T>,
}
impl<T> AllDomainAlt<T> {
    pub fn new() -> Self {
        AllDomainAlt { _marker: PhantomData }
    }
}
impl<T> Clone for AllDomainAlt<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<T> DomainAlt for AllDomainAlt<T> {
    type Carrier = T;
    fn check_compatible(&self, _other: &Self) -> bool { true }
    fn check_valid(&self, _val: &Self::Carrier) -> bool { true }
}


/// A Domain that unwraps a Data wrapper.
pub struct DataDomain<T> {
    pub form_domain: Box<dyn Domain<Carrier=T>>,
}
impl<T> DataDomain<T> {
    pub fn new(form_domain: impl Domain<Carrier=T> + 'static) -> DataDomain<T> {
        DataDomain { form_domain: Box::new(form_domain) }
    }
}
impl<T> Clone for DataDomain<T> {
    fn clone(&self) -> Self {
        DataDomain { form_domain: self.form_domain.box_clone() }
    }
}
impl<T: 'static> TraitObject for DataDomain<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static + Form> Domain for DataDomain<T> {
    type Carrier = Data;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.form_domain.check_compatible(&*o.form_domain))
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        let val = val.as_form();
        self.form_domain.check_valid(val)
    }
}

pub struct DataDomainAlt<D: DomainAlt> {
    pub form_domain: D,
}
impl<D: DomainAlt> DataDomainAlt<D> {
    pub fn new(form_domain: D) -> Self {
        DataDomainAlt { form_domain }
    }
}
impl<D: DomainAlt> Clone for DataDomainAlt<D> {
    fn clone(&self) -> Self {
        DataDomainAlt::new(self.form_domain.clone())
    }
}
impl<D: DomainAlt> DomainAlt for DataDomainAlt<D> where
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
    pub fn new(lower: Bound<T>, upper: Bound<T>) -> IntervalDomain<T> {
        IntervalDomain { lower, upper }
    }
}
impl<T: 'static + Clone> TraitObject for IntervalDomain<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static + Clone + PartialOrd> Domain for IntervalDomain<T> {
    type Carrier = T;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |e| e == self)
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

#[derive(Clone, PartialEq)]
pub struct IntervalDomainAlt<T> {
    pub lower: Bound<T>,
    pub upper: Bound<T>,
}
impl<T> IntervalDomainAlt<T> {
    pub fn new(lower: Bound<T>, upper: Bound<T>) -> Self {
        IntervalDomainAlt { lower, upper }
    }
}
impl<T: Clone + PartialOrd> DomainAlt for IntervalDomainAlt<T> {
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
pub struct PairDomain<T0, T1>(pub Box<dyn Domain<Carrier=T0>>, pub Box<dyn Domain<Carrier=T1>>);
impl<T0, T1> PairDomain<T0, T1> {
    pub fn new(element_domain0: impl Domain<Carrier=T0> + 'static, element_domain1: impl Domain<Carrier=T1> + 'static) -> PairDomain<T0, T1> {
        PairDomain(Box::new(element_domain0), Box::new(element_domain1))
    }
}
impl<T0: 'static, T1: 'static> Clone for PairDomain<T0, T1> {
    fn clone(&self) -> Self {
        PairDomain(self.0.box_clone(), self.1.box_clone())
    }
}
impl<T0: 'static, T1: 'static> TraitObject for PairDomain<T0, T1> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T0: 'static + PartialEq, T1: 'static + PartialEq> Domain for PairDomain<T0, T1> {
    type Carrier = (T0, T1);
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.0.check_compatible(&*o.0) && self.1.check_compatible(&*o.1))
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        self.0.check_valid(&val.0) && self.1.check_valid(&val.1)
    }
}

pub struct PairDomainAlt<D0: DomainAlt, D1: DomainAlt>(pub D0, pub D1);
impl<D0: DomainAlt, D1: DomainAlt> PairDomainAlt<D0, D1> {
    pub fn new(element_domain0: D0, element_domain1: D1) -> Self {
        PairDomainAlt(element_domain0, element_domain1)
    }
}
impl<D0: DomainAlt, D1: DomainAlt> Clone for PairDomainAlt<D0, D1> {
    fn clone(&self) -> Self {
        PairDomainAlt(self.0.clone(), self.1.clone())
    }
}
impl<D0: DomainAlt, D1: DomainAlt> DomainAlt for PairDomainAlt<D0, D1> {
    type Carrier = (D0::Carrier, D1::Carrier);
    fn check_compatible(&self, other: &Self) -> bool {
        self.0.check_compatible(&other.0) && self.1.check_compatible(&other.1)
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        self.0.check_valid(&val.0) && self.1.check_valid(&val.1)
    }
}


/// A Domain that contains maps of (homogeneous) values.
pub struct MapDomain<T> {
    pub element_domain: Box<dyn Domain<Carrier=T>>
}
impl<T: 'static> MapDomain<T> {
    pub fn new(element_domain: impl Domain<Carrier=T> + 'static) -> MapDomain<T> {
        MapDomain { element_domain: Box::new(element_domain) }
    }
    pub fn new_all() -> MapDomain<T> {
        Self::new(AllDomain::<T>::new())
    }
}
impl<T> Clone for MapDomain<T> {
    fn clone(&self) -> Self {
        MapDomain { element_domain: self.element_domain.box_clone() }
    }
}
impl<T: 'static> TraitObject for MapDomain<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static> Domain for MapDomain<T> {
    type Carrier = HashMap<String, T>;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domain.check_compatible(&*o.element_domain))
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|e| self.element_domain.check_valid(e.1))
    }
}

#[derive(Clone)]
pub struct MapDomainAlt<D: DomainAlt> {
    pub element_domain: D
}
impl<D: DomainAlt> MapDomainAlt<D> {
    pub fn new(element_domain: D) -> Self {
        MapDomainAlt { element_domain }
    }
}
impl<T> MapDomainAlt<AllDomainAlt<T>> {
    pub fn new_all() -> Self {
        Self::new(AllDomainAlt::<T>::new())
    }
}
impl<D: DomainAlt> DomainAlt for MapDomainAlt<D> {
    type Carrier = HashMap<String, D::Carrier>;
    fn check_compatible(&self, other: &Self) -> bool {
        self.element_domain.check_compatible(&other.element_domain)
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|e| self.element_domain.check_valid(e.1))
    }
}


/// A Domain that contains maps of heterogeneous values (wrapped in Data).
pub struct HeterogeneousMapDomain {
    pub element_domains: HashMap<String, Box<dyn Domain<Carrier=Data>>>,
}
impl HeterogeneousMapDomain {
    pub fn new(element_domains: HashMap<String, Box<dyn Domain<Carrier=Data>>>) -> HeterogeneousMapDomain {
        HeterogeneousMapDomain { element_domains }
    }
}
impl TraitObject for HeterogeneousMapDomain {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl Clone for HeterogeneousMapDomain {
    fn clone(&self) -> Self {
        let element_domains = self.element_domains.iter().map(|(k, v)| (k.clone(), v.box_clone())).collect();
        Self::new(element_domains)
    }
}
impl Domain for HeterogeneousMapDomain {
    type Carrier = HashMap<String, Data>;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        // TODO: Better MapDomain::check_compatible()
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domains.iter().all(|(k, v)| o.element_domains.get(k).map_or(false, |ov| v.check_compatible(ov.as_ref()))))
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|(k, v)| self.element_domains.get(k).map_or(false, |d| d.check_valid(v)))
    }
}


/// A Domain that contains vectors of (homogeneous) values.
pub struct VectorDomain<T> {
    pub element_domain: Box<dyn Domain<Carrier=T>>,
}
impl<T: 'static> VectorDomain<T> {
    pub fn new(element_domain: impl Domain<Carrier=T> + 'static) -> VectorDomain<T> {
        VectorDomain { element_domain: Box::new(element_domain) }
    }
    pub fn new_all() -> VectorDomain<T> {
        Self::new(AllDomain::<T>::new())
    }
}
impl<T> Clone for VectorDomain<T> {
    fn clone(&self) -> Self {
        VectorDomain { element_domain: self.element_domain.box_clone() }
    }
}
impl<T: 'static> TraitObject for VectorDomain<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static> Domain for VectorDomain<T> {
    type Carrier = Vec<T>;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domain.check_compatible(&*o.element_domain))
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|e| self.element_domain.check_valid(e))
    }
}

#[derive(Clone)]
pub struct VectorDomainAlt<D: DomainAlt> {
    pub element_domain: D,
}
impl<D: DomainAlt> VectorDomainAlt<D> {
    pub fn new(element_domain: D) -> Self {
        VectorDomainAlt { element_domain }
    }
}
impl<T> VectorDomainAlt<AllDomainAlt<T>> {
    pub fn new_all() -> Self {
        Self::new(AllDomainAlt::<T>::new())
    }
}
impl<D: DomainAlt> DomainAlt for VectorDomainAlt<D> {
    type Carrier = Vec<D::Carrier>;
    fn check_compatible(&self, other: &Self) -> bool {
        self.element_domain.check_compatible(&other.element_domain)
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|e| self.element_domain.check_valid(e))
    }
}
