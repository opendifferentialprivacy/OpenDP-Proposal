use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Bound;

use crate::core::DomainPtr;
use crate::data::{Data, Form, TraitObject};


/// A Domain that contains all members of the carrier type.
#[derive(PartialEq)]
pub struct AllDomainPtr<T> {
    _marker: PhantomData<T>,
}
impl<T> AllDomainPtr<T> {
    pub fn new() -> AllDomainPtr<T> {
        AllDomainPtr { _marker: PhantomData }
    }
}
impl<T> Clone for AllDomainPtr<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<T: 'static> TraitObject for AllDomainPtr<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static> DomainPtr for AllDomainPtr<T> {
    type Carrier = T;
    fn box_clone(&self) -> Box<dyn DomainPtr<Carrier=Self::Carrier>> {
        Box::new(Self::clone(self))
    }
    fn check_compatible(&self, _other: &dyn DomainPtr<Carrier=Self::Carrier>) -> bool {
        true
    }
    fn check_valid(&self, _val: &Self::Carrier) -> bool { true }
}


/// A Domain that unwraps a Data wrapper.
pub struct DataDomainPtr<T> {
    pub form_domain: Box<dyn DomainPtr<Carrier=T>>,
}
impl<T> DataDomainPtr<T> {
    pub fn new(form_domain: impl DomainPtr<Carrier=T> + 'static) -> DataDomainPtr<T> {
        DataDomainPtr { form_domain: Box::new(form_domain) }
    }
}
impl<T> Clone for DataDomainPtr<T> {
    fn clone(&self) -> Self {
        DataDomainPtr { form_domain: self.form_domain.box_clone() }
    }
}
impl<T: 'static> TraitObject for DataDomainPtr<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static + Form> DomainPtr for DataDomainPtr<T> {
    type Carrier = Data;
    fn box_clone(&self) -> Box<dyn DomainPtr<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn DomainPtr<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.form_domain.check_compatible(&*o.form_domain))
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        let val = val.as_form();
        self.form_domain.check_valid(val)
    }
}


/// A Domain that contains all the values in an interval.
#[derive(Clone, PartialEq)]
pub struct IntervalDomainPtr<T> {
    pub lower: Bound<T>,
    pub upper: Bound<T>,
}
impl<T> IntervalDomainPtr<T> {
    pub fn new(lower: Bound<T>, upper: Bound<T>) -> IntervalDomainPtr<T> {
        IntervalDomainPtr { lower, upper }
    }
}
impl<T: 'static + Clone> TraitObject for IntervalDomainPtr<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static + Clone + PartialOrd> DomainPtr for IntervalDomainPtr<T> {
    type Carrier = T;
    fn box_clone(&self) -> Box<dyn DomainPtr<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn DomainPtr<Carrier=Self::Carrier>) -> bool {
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


/// A Domain that contains pairs of values.
pub struct PairDomainPtr<T0, T1>(pub Box<dyn DomainPtr<Carrier=T0>>, pub Box<dyn DomainPtr<Carrier=T1>>);
impl<T0, T1> PairDomainPtr<T0, T1> {
    pub fn new(element_domain0: impl DomainPtr<Carrier=T0> + 'static, element_domain1: impl DomainPtr<Carrier=T1> + 'static) -> PairDomainPtr<T0, T1> {
        PairDomainPtr(Box::new(element_domain0), Box::new(element_domain1))
    }
}
impl<T0: 'static, T1: 'static> Clone for PairDomainPtr<T0, T1> {
    fn clone(&self) -> Self {
        PairDomainPtr(self.0.box_clone(), self.1.box_clone())
    }
}
impl<T0: 'static, T1: 'static> TraitObject for PairDomainPtr<T0, T1> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T0: 'static + PartialEq, T1: 'static + PartialEq> DomainPtr for PairDomainPtr<T0, T1> {
    type Carrier = (T0, T1);
    fn box_clone(&self) -> Box<dyn DomainPtr<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn DomainPtr<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.0.check_compatible(&*o.0) && self.1.check_compatible(&*o.1))
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        self.0.check_valid(&val.0) && self.1.check_valid(&val.1)
    }
}


/// A Domain that contains maps of (homogeneous) values.
pub struct MapDomainPtr<T> {
    pub element_domain: Box<dyn DomainPtr<Carrier=T>>
}
impl<T: 'static> MapDomainPtr<T> {
    pub fn new(element_domain: impl DomainPtr<Carrier=T> + 'static) -> MapDomainPtr<T> {
        MapDomainPtr { element_domain: Box::new(element_domain) }
    }
    pub fn new_all() -> MapDomainPtr<T> {
        Self::new(AllDomainPtr::<T>::new())
    }
}
impl<T> Clone for MapDomainPtr<T> {
    fn clone(&self) -> Self {
        MapDomainPtr { element_domain: self.element_domain.box_clone() }
    }
}
impl<T: 'static> TraitObject for MapDomainPtr<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static> DomainPtr for MapDomainPtr<T> {
    type Carrier = HashMap<String, T>;
    fn box_clone(&self) -> Box<dyn DomainPtr<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn DomainPtr<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domain.check_compatible(&*o.element_domain))
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|e| self.element_domain.check_valid(e.1))
    }
}


/// A Domain that contains maps of heterogeneous values (wrapped in Data).
pub struct HeterogeneousMapDomainPtr {
    pub element_domains: HashMap<String, Box<dyn DomainPtr<Carrier=Data>>>,
}
impl HeterogeneousMapDomainPtr {
    pub fn new(element_domains: HashMap<String, Box<dyn DomainPtr<Carrier=Data>>>) -> HeterogeneousMapDomainPtr {
        HeterogeneousMapDomainPtr { element_domains }
    }
}
impl TraitObject for HeterogeneousMapDomainPtr {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl Clone for HeterogeneousMapDomainPtr {
    fn clone(&self) -> Self {
        let element_domains = self.element_domains.iter().map(|(k, v)| (k.clone(), v.box_clone())).collect();
        Self::new(element_domains)
    }
}
impl DomainPtr for HeterogeneousMapDomainPtr {
    type Carrier = HashMap<String, Data>;
    fn box_clone(&self) -> Box<dyn DomainPtr<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn DomainPtr<Carrier=Self::Carrier>) -> bool {
        // TODO: Better MapDomain::check_compatible()
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domains.iter().all(|(k, v)| o.element_domains.get(k).map_or(false, |ov| v.check_compatible(ov.as_ref()))))
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|(k, v)| self.element_domains.get(k).map_or(false, |d| d.check_valid(v)))
    }
}


/// A Domain that contains vectors of (homogeneous) values.
pub struct VectorDomainPtr<T> {
    pub element_domain: Box<dyn DomainPtr<Carrier=T>>,
}
impl<T: 'static> VectorDomainPtr<T> {
    pub fn new(element_domain: impl DomainPtr<Carrier=T> + 'static) -> VectorDomainPtr<T> {
        VectorDomainPtr { element_domain: Box::new(element_domain) }
    }
    pub fn new_all() -> VectorDomainPtr<T> {
        Self::new(AllDomainPtr::<T>::new())
    }
}
impl<T> Clone for VectorDomainPtr<T> {
    fn clone(&self) -> Self {
        VectorDomainPtr { element_domain: self.element_domain.box_clone() }
    }
}
impl<T: 'static> TraitObject for VectorDomainPtr<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static> DomainPtr for VectorDomainPtr<T> {
    type Carrier = Vec<T>;
    fn box_clone(&self) -> Box<dyn DomainPtr<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn DomainPtr<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domain.check_compatible(&*o.element_domain))
    }
    fn check_valid(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|e| self.element_domain.check_valid(e))
    }
}
