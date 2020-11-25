use std::any::Any;
use std::ops::Bound;
use std::marker::PhantomData;


pub struct Generic {
    concrete: Box<dyn Concrete>,
}
impl Generic {
    pub fn new<T: 'static + Concrete>(concrete: T) -> Generic {
        Generic { concrete: Box::new(concrete) }
    }
    pub fn as_concrete<T: 'static + Concrete>(&self) -> Option<&T> {
        self.concrete.as_any().downcast_ref()
    }
}
impl Clone for Generic {
    fn clone(&self) -> Self {
        Generic { concrete: self.concrete.box_clone() }
    }
}

pub trait Concrete {
    fn as_any(&self) -> &dyn Any;
    fn box_clone(&self) -> Box<dyn Concrete>;
}


pub trait Domain {
    fn as_any(&self) -> &dyn Any;
    fn check_compatible(&self, other: &dyn Domain) -> bool;
    fn check_valid(&self, val: &Generic) -> bool;
}

pub trait DomainImpl<T> {
    fn check_valid_impl(&self, val: &T) -> bool;
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
impl<T: 'static + Concrete + PartialOrd> Domain for IntervalDomain<T> {
    fn as_any(&self) -> &dyn Any { self }
    fn check_compatible(&self, other: &dyn Domain) -> bool {
        // TODO: Make this the default impl after making Domain be PartialEq.
        other.as_any().downcast_ref::<Self>().map_or(false, |e| e == self)
    }
    fn check_valid(&self, val: &Generic) -> bool {
        val.as_concrete::<T>().map_or(false, |v| self.check_valid_impl(v))
    }
}
impl<T: 'static + Concrete + PartialOrd> DomainImpl<T> for IntervalDomain<T> {
    fn check_valid_impl(&self, val: &T) -> bool {
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
    element_domain: Box<dyn Domain>,
    _marker: PhantomData<T>,
}
impl<T: 'static> VectorDomain<T> {
    pub fn new(element_domain: Box<dyn Domain>) -> VectorDomain<T> {
        VectorDomain { element_domain, _marker: PhantomData }
    }
}
impl<T: 'static> Domain for VectorDomain<T> where
    T: Concrete + Clone,
    Vec<T>: Concrete {
    fn as_any(&self) -> &dyn Any { self }
    fn check_compatible(&self, other: &dyn Domain) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domain.check_compatible(&*o.element_domain))
    }
    fn check_valid(&self, val: &Generic) -> bool {
        val.as_concrete::<Vec<T>>().map_or(false, |v| self.check_valid_impl(v))
    }
}
impl<T: 'static + Concrete> DomainImpl<Vec<T>> for VectorDomain<T> where
    T: Concrete + Clone,
    Vec<T>: Concrete {
    fn check_valid_impl(&self, val: &Vec<T>) -> bool {
        let val: Vec<Generic> = val.into_iter().map(|e| Generic::new(e.clone())).collect();
        val.iter().all(|e| self.element_domain.check_valid(e))
    }
}
