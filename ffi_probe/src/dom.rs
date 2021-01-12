use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Bound;

use crate::core::{Domain, DomainImpl, DomainPtr};
use crate::data::{Data, Element, Form, TraitObject};

/// A Domain that contains all members of the carrier type.
#[derive(Clone, PartialEq)]
pub struct AllDomain<T> {
    _marker: PhantomData<T>,
}
impl<T> AllDomain<T> {
    pub fn new() -> AllDomain<T> {
        AllDomain { _marker: PhantomData }
    }
}
impl<T: 'static + Form + Clone> TraitObject for AllDomain<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static + Form + Clone> Domain for AllDomain<T> {
    fn box_clone(&self) -> Box<dyn Domain> { Box::new(self.clone()) }
    fn check_compatible(&self, _other: &dyn Domain) -> bool {
        true
    }
    fn check_valid(&self, val: &Data) -> bool {
        let val: &T = val.as_form();
        self.check_valid_impl(val)
    }
}
impl<T: 'static + Form + Clone> DomainImpl for AllDomain<T> {
    type Carrier = T;
    fn check_valid_impl(&self, _val: &Self::Carrier) -> bool {
        true
    }
}

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
pub struct DataDomain {
    pub form_domain: Box<dyn Domain>,
}
impl DataDomain {
    pub fn new(form_domain: impl Domain + 'static) -> DataDomain {
        DataDomain { form_domain: Box::new(form_domain) }
    }
    pub fn new_box(form_domain: Box<dyn Domain>) -> DataDomain {
        DataDomain { form_domain: form_domain }
    }
}
impl TraitObject for DataDomain {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl Clone for DataDomain {
    fn clone(&self) -> Self {
        DataDomain { form_domain: self.form_domain.box_clone() }
    }
}
impl Domain for DataDomain {
    fn box_clone(&self) -> Box<dyn Domain> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.form_domain.check_compatible(&*o.form_domain))
    }
    fn check_valid(&self, val: &Data) -> bool {
        self.check_valid_impl(val)
    }
}
impl DomainImpl for DataDomain {
    type Carrier = Data;
    fn check_valid_impl(&self, val: &Self::Carrier) -> bool {
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
impl<T: 'static + Element + Clone + PartialOrd> Domain for IntervalDomain<T> {
    fn box_clone(&self) -> Box<dyn Domain> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |e| e == self)
    }
    fn check_valid(&self, val: &Data) -> bool {
        let val: &T = val.as_form();
        self.check_valid_impl(val)
    }
}
impl<T: 'static + Element + Clone + PartialOrd> DomainImpl for IntervalDomain<T> {
    type Carrier = T;
    fn check_valid_impl(&self, val: &Self::Carrier) -> bool {
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
pub struct PairDomain<T>(Box<dyn Domain>, Box<dyn Domain>, PhantomData<T>);
impl<T: 'static + Element> PairDomain<T> {
    pub fn new(element_domain0: Box<dyn Domain>, element_domain1: Box<dyn Domain>) -> PairDomain<T> {
        PairDomain(element_domain0, element_domain1, PhantomData)
    }
}
impl<T: 'static + Element> TraitObject for PairDomain<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static + Element + Clone + PartialEq> Clone for PairDomain<T> {
    fn clone(&self) -> Self {
        PairDomain::new(self.0.box_clone(), self.1.box_clone())
    }
}
impl<T: 'static + Element + Clone + PartialEq> Domain for PairDomain<T> {
    fn box_clone(&self) -> Box<dyn Domain> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.0.check_compatible(&*o.0) && self.1.check_compatible(&*o.1))
    }
    fn check_valid(&self, val: &Data) -> bool {
        let val: &(T, Data) = val.as_form();
        self.check_valid_impl(val)
    }
}
impl<T: 'static + Element + Clone + PartialEq> DomainImpl for PairDomain<T> {
    type Carrier = (T, Data);
    fn check_valid_impl(&self, val: &Self::Carrier) -> bool {
        let val = (Data::new(val.0.clone()), &val.1);
        self.0.check_valid(&val.0) && self.1.check_valid(&val.1)
    }
}

pub struct PairDomainPtr<T0, T1>(Box<dyn DomainPtr<Carrier=T0>>, Box<dyn DomainPtr<Carrier=T1>>);
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
pub struct MapDomain<T> {
    pub element_domain: Box<dyn Domain>,
    _marker: PhantomData<T>,
}
impl<T: 'static + Element + Clone + PartialEq> MapDomain<T> {
    pub fn new(element_domain: impl Domain + 'static) -> MapDomain<T> {
        MapDomain { element_domain: Box::new(element_domain), _marker: PhantomData }
    }
    pub fn new_all() -> MapDomain<T> {
        Self::new(AllDomain::<T>::new())
    }
}
impl<T: 'static + Element> TraitObject for MapDomain<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static + Element + Clone + PartialEq> Clone for MapDomain<T> {
    fn clone(&self) -> Self {
        MapDomain { element_domain: self.element_domain.box_clone(), _marker: PhantomData }
    }
}
impl<T: 'static + Element + Clone + PartialEq> Domain for MapDomain<T> {
    fn box_clone(&self) -> Box<dyn Domain> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domain.check_compatible(&*o.element_domain))
    }
    fn check_valid(&self, val: &Data) -> bool {
        let val: &HashMap<String, T> = val.as_form();
        self.check_valid_impl(val)
    }
}
impl<T: 'static + Element + Clone + PartialEq> DomainImpl for MapDomain<T> {
    type Carrier = HashMap<String, T>;
    fn check_valid_impl(&self, val: &Self::Carrier) -> bool {
        // TODO: Implement more efficient delegation to element domain (avoid wrapping elements).
        let val: HashMap<String, Data> = val.iter().map(|(k, v)| (k.clone(), Data::new(v.clone()))).collect();
        val.iter().all(|e| self.element_domain.check_valid(e.1))
    }
}

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
pub struct HeterogeneousMapDomain {
    pub element_domains: HashMap<String, Box<dyn Domain>>,
}
impl HeterogeneousMapDomain {
    pub fn new(element_domains: HashMap<String, Box<dyn Domain>>) -> HeterogeneousMapDomain {
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
    fn box_clone(&self) -> Box<dyn Domain> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain) -> bool {
        // TODO: Better MapDomain::check_compatible()
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domains.iter().all(|(k, v)| o.element_domains.get(k).map_or(false, |ov| v.check_compatible(ov.as_ref()))))
    }
    fn check_valid(&self, val: &Data) -> bool {
        let val: &HashMap<String, Data> = val.as_form();
        self.check_valid_impl(val)
    }
}
impl DomainImpl for HeterogeneousMapDomain {
    type Carrier = HashMap<String, Data>;
    fn check_valid_impl(&self, val: &Self::Carrier) -> bool {
        // TODO: Implement more efficient delegation to element domain (avoid wrapping elements).
        let val: HashMap<String, Data> = val.iter().map(|(k, v)| (k.clone(), Data::new(v.clone()))).collect();
        val.iter().all(|(k, v)| self.element_domains.get(k).map_or(false, |d| d.check_valid(v)))
    }
}

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
pub struct VectorDomain<T> {
    pub element_domain: Box<dyn Domain>,
    _marker: PhantomData<T>,
}
impl<T: 'static + Element + Clone + PartialEq> VectorDomain<T> {
    pub fn new(element_domain: impl Domain + 'static) -> VectorDomain<T> {
        VectorDomain { element_domain: Box::new(element_domain), _marker: PhantomData }
    }
    pub fn new_all() -> VectorDomain<T> {
        Self::new(AllDomain::<T>::new())
    }
}
impl<T: 'static + Element> TraitObject for VectorDomain<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static + Element + Clone + PartialEq> Clone for VectorDomain<T> {
    fn clone(&self) -> Self {
        VectorDomain { element_domain: self.element_domain.box_clone(), _marker: PhantomData }
    }
}
impl<T: 'static + Element + Clone + PartialEq> Domain for VectorDomain<T> {
    fn box_clone(&self) -> Box<dyn Domain> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domain.check_compatible(&*o.element_domain))
    }
    fn check_valid(&self, val: &Data) -> bool {
        let val: &Vec<T> = val.as_form();
        self.check_valid_impl(val)
    }
}
impl<T: 'static + Element + Clone + PartialEq> DomainImpl for VectorDomain<T> {
    type Carrier = Vec<T>;
    fn check_valid_impl(&self, val: &Self::Carrier) -> bool {
        // TODO: Implement more efficient delegation to element domain (avoid wrapping elements).
        let val: Vec<Data> = val.iter().map(|e| Data::new(e.clone())).collect();
        val.iter().all(|e| self.element_domain.check_valid(e))
    }
}

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
