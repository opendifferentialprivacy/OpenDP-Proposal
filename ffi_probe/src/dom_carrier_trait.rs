use std::any::Any;
use std::ops::Bound;
use std::marker::PhantomData;


pub struct Generic {
    concrete: Box<dyn Any>,
}
impl Generic {
    pub fn new<T: 'static + Concrete>(concrete: T) -> Generic {
        Generic { concrete: Box::new(concrete) }
    }
    pub fn as_concrete<T: 'static + Concrete>(&self) -> Option<&T> {
        self.concrete.downcast_ref()
    }
}

pub trait Concrete {
    fn as_any(&self) -> &dyn Any;
}

pub trait Carry: Concrete {
    type Carrier;
}


pub trait Domain: Carry {
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool;
    fn check_valid(&self, val: &Generic) -> bool where
        Self::Carrier: 'static + Concrete {
        val.as_concrete::<Self::Carrier>().map_or(false, |v| self.check_valid_impl(v))
    }
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
impl<T: 'static> Concrete for IntervalDomain<T> {
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static> Carry for IntervalDomain<T> {
    type Carrier = T;
}
impl<T: 'static + PartialOrd> Domain for IntervalDomain<T> {
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
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
impl<T: 'static> VectorDomain<T> {
    pub fn new(element_domain: Box<dyn Domain<Carrier=T>>) -> VectorDomain<T> {
        VectorDomain { element_domain, _marker: PhantomData }
    }
}
impl<T: 'static> Concrete for VectorDomain<T> {
    fn as_any(&self) -> &dyn Any { self }
}
impl<T: 'static> Carry for VectorDomain<T> {
    type Carrier = Vec<T>;
}
impl<T: 'static> Domain for VectorDomain<T> {
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |o| self.element_domain.check_compatible(&*o.element_domain))
    }
    fn check_valid_impl(&self, val: &Vec<T>) -> bool {
        val.iter().all(|e| self.element_domain.check_valid_impl(e))
    }
}
