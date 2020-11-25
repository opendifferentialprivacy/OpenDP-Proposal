use crate::data::{Data, Form, Primitive};
use std::any::Any;
use std::ops::Bound;


pub trait Domain {
    type Carrier;
    fn as_any(&self) -> &dyn Any;
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool;
    fn check_valid(&self, val: &Data) -> bool;
    fn check_valid_carrier(&self, val: &Self::Carrier) -> bool;
}

pub trait DomainImpl {
    type Carrier;
    fn check_valid_impl(&self, val: &Self::Carrier) -> bool;
}


#[derive(PartialEq)]
pub struct IntervalDomain<T> {
    lower: Bound<T>,
    upper: Bound<T>,
}
impl<T> IntervalDomain<T> {
    pub fn new(lower: Bound<T>, upper: Bound<T>) -> IntervalDomain<T> {
        IntervalDomain { lower, upper }
    }
}
impl<T: Primitive + Clone + PartialOrd> DomainImpl for IntervalDomain<T> {
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
impl<T: 'static + Primitive + Clone + PartialOrd> Domain for IntervalDomain<T> {
    type Carrier = T;
    fn as_any(&self) -> &dyn Any { self }
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |e| e == self)
    }
    fn check_valid(&self, val: &Data) -> bool {
        let val: &Self::Carrier = val.as_form();
        self.check_valid_carrier(val)
    }
    fn check_valid_carrier(&self, val: &Self::Carrier) -> bool {
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
}
impl<T: 'static + Form + Clone + PartialEq> VectorDomain<T> {
    pub fn new(element_domain: Box<dyn Domain<Carrier=T>>) -> VectorDomain<T> {
        VectorDomain { element_domain }
    }
}
impl<T: 'static + Form + Clone + PartialEq> DomainImpl for VectorDomain<T> {
    type Carrier = Vec<T>;
    fn check_valid_impl(&self, val: &Vec<T>) -> bool {
        let val: Vec<Data> = val.into_iter().map(|e| Data::new(e.clone())).collect();
        val.iter().all(|e| self.element_domain.check_valid(e))
    }
}
impl<T: 'static + Form + Clone + PartialEq> Domain for VectorDomain<T> {
    type Carrier = Vec<T>;
    fn as_any(&self) -> &dyn Any { self }
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |other| self.element_domain.check_compatible(&*other.element_domain))
    }
    fn check_valid(&self, val: &Data) -> bool {
        let val: &Self::Carrier = val.as_form();
        self.check_valid_impl(val)
    }
    fn check_valid_carrier(&self, val: &Self::Carrier) -> bool {
        val.iter().all(|e| self.element_domain.check_valid_carrier(e))
    }
}
