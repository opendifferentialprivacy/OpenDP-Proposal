use std::any::Any;
use std::marker::PhantomData;
use std::ops::Bound;

use crate::core::Domain;
use crate::data::{Form, TraitObject};


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
    type Carrier = T;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, _other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        true
    }
    fn check_valid_impl(&self, _val: &Self::Carrier) -> bool {
        true
    }
}


#[derive(Clone, PartialEq)]
pub struct IntervalDomain<T> {
    lower: Bound<T>,
    upper: Bound<T>,
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
impl<T: 'static + Form + Clone + PartialOrd> Domain for IntervalDomain<T> {
    type Carrier = T;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        // TODO: Accept enclosing intervals.
        other.as_any().downcast_ref::<Self>().map_or(false, |e| e == self)
    }
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


pub struct VectorDomain<T> {
    element_domain: Box<dyn Domain<Carrier=T>>,
    _marker: PhantomData<T>,
}
impl<T: 'static + Form> VectorDomain<T> {
    pub fn new(element_domain: Box<dyn Domain<Carrier=T>>) -> VectorDomain<T> {
        VectorDomain { element_domain, _marker: PhantomData }
    }
}
impl<T: 'static + Form> TraitObject for VectorDomain<T> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static + Form> Clone for VectorDomain<T> {
    fn clone(&self) -> Self {
        VectorDomain::new(self.element_domain.box_clone())
    }
}
impl<T: 'static + Form + Clone + PartialEq> Domain for VectorDomain<T> {
    type Carrier = Vec<T>;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>> { Box::new(self.clone()) }
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domain.check_compatible(&*o.element_domain))
    }
    fn check_valid_impl(&self, val: &Vec<T>) -> bool {
        val.iter().all(|e| self.element_domain.check_valid_impl(e))
    }
}
