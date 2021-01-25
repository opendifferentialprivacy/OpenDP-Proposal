use std::rc::Rc;

use crate::core::dummy::{dummy_relation, DummyDomain, DummyMeasure, DummyMetric};
use crate::core::ffi::FfiGlue;
use crate::ffi_utils;

// BUILDING BLOCKS
pub trait Domain: Clone {
    type Carrier;
    fn check_compatible(&self, other: &Self) -> bool;
    fn check_valid(&self, val: &Self::Carrier) -> bool;
}

#[derive(Clone)]
pub struct Function<I, O> {
    function: Rc<dyn Fn(*const I) -> Box<O>>
}

impl<I, O> Function<I, O> {
    pub fn new(function: impl Fn(&I) -> O + 'static) -> Self {
        let function = move |arg: *const I| -> Box<O> {
            let arg = ffi_utils::as_ref(arg);
            let res = function(arg);
            Box::new(res)
        };
        let function = Rc::new(function);
        Function { function }
    }

    pub fn eval(&self, arg: &I) -> O {
        let arg = arg as *const I;
        let res = (self.function)(arg);
        *res
    }

    pub fn eval_ffi(&self, arg: &Box<I>) -> Box<O> {
        let arg = arg.as_ref() as *const I;
        (self.function)(arg)
    }
}

impl<I: 'static, O: 'static> Function<I, O> {
    pub fn make_chain<X: 'static>(function1: &Function<X, O>, function0: &Function<I, X>) -> Function<I, O> {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        let function = move |arg: *const I| -> Box<O> {
            let res0 = function0(arg);
            function1(&*res0)
        };
        let function = Rc::new(function);
        Function { function }
    }
}

impl<I: 'static, OA: 'static, OB: 'static> Function<I, (Box<OA>, Box<OB>)> {
    pub fn make_composition(function0: &Function<I, OA>, function1: &Function<I, OB>) -> Function<I, (Box<OA>, Box<OB>)> {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        let function = move |arg: *const I| -> Box<(Box<OA>, Box<OB>)> {
            let res0 = function0(arg);
            let res1 = function1(arg);
            Box::new((res0, res1))
        };
        let function = Rc::new(function);
        Function { function }
    }
}

pub trait Metric: Clone {
    type Distance;
}

pub trait Measure: Clone {
    type Distance;
}

#[derive(Clone)]
pub struct Relation<I, O> {
    relation: Rc<dyn Fn(*const I, *const O) -> bool>
}
impl<I, O> Relation<I, O> {
    pub fn new(relation: impl Fn(&I, &O) -> bool + 'static) -> Self {
        let relation = move |input_distance: *const I, output_distance: *const O| -> bool {
            let input_distance = ffi_utils::as_ref(input_distance);
            let output_distance = ffi_utils::as_ref(output_distance);
            relation(input_distance, output_distance)
        };
        let relation = Rc::new(relation);
        Relation { relation }
    }
    pub fn eval(&self, input_distance: &I, output_distance: &O) -> bool {
        let input_distance = input_distance as *const I;
        let output_distance = output_distance as *const O;
        (self.relation)(input_distance, output_distance)
    }
    pub fn eval_ffi(&self, input_distance: *const I, output_distance: *const O) -> bool {
        (self.relation)(input_distance, output_distance)
    }
}


// MEASUREMENTS & TRANSFORMATIONS
pub struct Measurement<ID: Domain, OD: Domain, IM: Metric=DummyMetric<()>, OM: Measure=DummyMeasure<()>> {
    pub input_domain: Box<ID>,
    pub output_domain: Box<OD>,
    pub function: Function<ID::Carrier, OD::Carrier>,
    pub input_metric: IM,
    pub output_measure: OM,
    pub privacy_relation: Relation<IM::Distance, OM::Distance>,
}

impl<ID: Domain, OD: Domain> Measurement<ID, OD> {
    pub fn new(input_domain: ID, output_domain: OD, function: impl Fn(&ID::Carrier) -> OD::Carrier + 'static) -> Self {
        let function = Function::new(function);
        let input_metric = dummy::DummyMetric::new();
        let output_measure = dummy::DummyMeasure::new();
        let privacy_relation = dummy::dummy_relation();
        Self::new_all(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
    }
}

impl<ID: Domain, OD: Domain, IM: Metric, OM: Measure> Measurement<ID, OD, IM, OM> {
    pub fn new_all(
        input_domain: ID,
        output_domain: OD,
        function: Function<ID::Carrier, OD::Carrier>,
        input_metric: IM,
        output_measure: OM,
        privacy_relation: Relation<IM::Distance, OM::Distance>,
    ) -> Self {
        let input_domain = Box::new(input_domain);
        let output_domain = Box::new(output_domain);
        Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
    }
}

pub struct Transformation<ID: Domain, OD: Domain, IM: Metric=DummyMetric<()>, OM: Metric=DummyMetric<()>> {
    pub input_domain: Box<ID>,
    pub output_domain: Box<OD>,
    pub function: Function<ID::Carrier, OD::Carrier>,
    pub input_metric: IM,
    pub output_metric: OM,
    pub stability_relation: Relation<IM::Distance, OM::Distance>,
}

impl<ID: Domain, OD: Domain> Transformation<ID, OD> {
    pub fn new(input_domain: ID, output_domain: OD, function: impl Fn(&ID::Carrier) -> OD::Carrier + 'static) -> Self {
        let function = Function::new(function);
        let input_metric = dummy::DummyMetric::new();
        let output_metric = dummy::DummyMetric::new();
        let stability_relation = dummy::dummy_relation();
        Self::new_all(input_domain, output_domain, function, input_metric, output_metric, stability_relation)
    }
}

impl<ID: Domain, OD: Domain, IM: Metric, OM: Metric> Transformation<ID, OD, IM, OM> {
    pub fn new_all(
        input_domain: ID,
        output_domain: OD,
        function: Function<ID::Carrier, OD::Carrier>,
        input_metric: IM,
        output_metric: OM,
        stability_relation: Relation<IM::Distance, OM::Distance>,
    ) -> Self {
        let input_domain = Box::new(input_domain);
        let output_domain = Box::new(output_domain);
        Transformation { input_domain, output_domain, function, input_metric, output_metric, stability_relation }
    }
}


// CHAINING & COMPOSITION
pub fn make_chain_mt<ID, XD, OD, IM, XM, OM>(measurement1: &Measurement<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>) -> Measurement<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: Metric, XM: Metric, OM: Measure {
    let input_glue = FfiGlue::<ID>::new_from_type();
    let x_glue = FfiGlue::<XD>::new_from_type();
    let output_glue = FfiGlue::<OD>::new_from_type();
    make_chain_mt_core(measurement1, transformation0, &input_glue, &x_glue, &output_glue)
}

fn make_chain_mt_core<ID, XD, OD, IM, XM, OM>(measurement1: &Measurement<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>, input_glue: &FfiGlue<ID>, x_glue: &FfiGlue<XD>, output_glue: &FfiGlue<OD>) -> Measurement<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: Metric, XM: Metric, OM: Measure {
    assert!((x_glue.eq)(&transformation0.output_domain, &measurement1.input_domain));
    let input_domain = (input_glue.clone_)(&transformation0.input_domain);
    let output_domain = (output_glue.clone_)(&measurement1.output_domain);
    let function = Function::make_chain(&measurement1.function, &transformation0.function);
    let input_metric = transformation0.input_metric.clone();
    let output_measure = measurement1.output_measure.clone();
    let privacy_relation = dummy_relation();
    Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
}

pub fn make_chain_tt<ID, XD, OD, IM, XM, OM>(transformation1: &Transformation<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>) -> Transformation<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: Metric, XM: Metric, OM: Metric {
    let input_glue = FfiGlue::<ID>::new_from_type();
    let x_glue = FfiGlue::<XD>::new_from_type();
    let output_glue = FfiGlue::<OD>::new_from_type();
    make_chain_tt_core(transformation1, transformation0, &input_glue, &x_glue, &output_glue)
}

fn make_chain_tt_core<ID, XD, OD, IM, XM, OM>(transformation1: &Transformation<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>, input_glue: &FfiGlue<ID>, x_glue: &FfiGlue<XD>, output_glue: &FfiGlue<OD>) -> Transformation<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: Metric, XM: Metric, OM: Metric {
    assert!((x_glue.eq)(&transformation0.output_domain, &transformation1.input_domain));
    let input_domain = (input_glue.clone_)(&transformation0.input_domain);
    let output_domain = (output_glue.clone_)(&transformation1.output_domain);
    let function = Function::make_chain(&transformation1.function, &transformation0.function);
    let input_metric = transformation0.input_metric.clone();
    let output_metric = transformation1.output_metric.clone();
    let stability_relation = dummy_relation();
    Transformation { input_domain, output_domain, function, input_metric, output_metric, stability_relation }
}

pub fn make_composition<ID, OD0, OD1, IM, OM>(measurement0: &Measurement<ID, OD0, IM, OM>, measurement1: &Measurement<ID, OD1, IM, OM>) -> Measurement<ID, DummyDomain<(Box<OD0::Carrier>, Box<OD1::Carrier>)>, IM, OM> where
    ID: 'static + Domain, OD0: 'static + Domain, OD1: 'static + Domain, IM: Metric, OM: Measure {
    let input_glue = FfiGlue::<ID>::new_from_type();
    let output_glue0 = FfiGlue::<OD0>::new_from_type();
    let output_glue1 = FfiGlue::<OD1>::new_from_type();
    make_composition_core(measurement0, measurement1, &input_glue, &output_glue0, &output_glue1)
}

fn make_composition_core<ID, OD0, OD1, IM, OM>(measurement0: &Measurement<ID, OD0, IM, OM>, measurement1: &Measurement<ID, OD1, IM, OM>, input_glue: &FfiGlue<ID>, _output_glue0: &FfiGlue<OD0>, _output_glue1: &FfiGlue<OD1>) -> Measurement<ID, DummyDomain<(Box<OD0::Carrier>, Box<OD1::Carrier>)>, IM, OM> where
    ID: 'static + Domain, OD0: 'static + Domain, OD1: 'static + Domain, IM: Metric, OM: Measure {
    assert!((input_glue.eq)(&measurement0.input_domain, &measurement1.input_domain));
    let input_domain = (input_glue.clone_)(&measurement0.input_domain);
    // TODO: Figure out output_domain for composition.
    let output_domain = Box::new(dummy::DummyDomain::new());
    let function = Function::make_composition(&measurement0.function, &measurement1.function);
    // TODO: Figure out input_metric for composition.
    let input_metric = measurement0.input_metric.clone();
    // TODO: Figure out output_measure for composition.
    let output_measure = measurement0.output_measure.clone();
    let privacy_relation = dummy_relation();
    Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
}


// FFI BINDINGS
pub(crate) mod ffi {
    use std::intrinsics::transmute;
    use std::os::raw::c_char;

    use crate::ffi_utils;
    use crate::mono::Type;

    use super::*;

    pub struct FfiObject {
        pub type_: Type,
        pub value: Box<()>,
    }

    impl FfiObject {
        pub fn new_typed(type_: Type, value: Box<()>) -> *mut FfiObject {
            let object = FfiObject { type_, value };
            ffi_utils::into_raw(object)
        }

        pub fn new<T: 'static>(value: T) -> *mut FfiObject {
            let type_ = Type::new::<T>();
            let value = ffi_utils::into_box(value);
            Self::new_typed(type_, value)
        }

        pub fn into_owned<T>(self) -> T {
            // TODO: Check T against self.type_.
            let value = Box::into_raw(self.value) as *mut T;
            ffi_utils::into_owned(value)
        }

        pub fn as_ref<T>(&self) -> &T {
            // TODO: Check type.
            let value = self.value.as_ref() as *const () as *const T;
            let value = unsafe { value.as_ref() };
            value.unwrap()
        }
    }

    #[derive(Clone)]
    pub struct FfiGlue<D: Domain> {
        pub type_: Type,
        pub carrier: Type,
        pub eq: Rc<dyn Fn(&Box<D>, &Box<D>) -> bool>,
        pub clone_: Rc<dyn Fn(&Box<D>) -> Box<D>>,
    }
    impl<D: 'static + Domain> FfiGlue<D> {
        pub fn new_from_type() -> Self {
            let type_ = Type::new::<D>();
            let carrier = Type::new::<D::Carrier>();
            let eq = |d0: &Box<D>, d1: &Box<D>| {
                d0.check_compatible(&d1)
            };
            let eq = Rc::new(eq);
            let clone_ = |d: &Box<D>| {
                d.clone()
            };
            let clone_ = Rc::new(clone_);
            Self::new(type_, carrier, eq, clone_)
        }
        pub fn new(type_: Type, carrier: Type, eq: Rc<dyn Fn(&Box<D>, &Box<D>) -> bool>, clone_: Rc<dyn Fn(&Box<D>) -> Box<D>>) -> Self {
            FfiGlue { type_, carrier, eq, clone_ }
        }
    }

    #[derive(Clone)]
    pub struct FfiDomain;
    impl Domain for FfiDomain {
        type Carrier = ();
        fn check_compatible(&self, _other: &Self) -> bool { unimplemented!() }
        fn check_valid(&self, _val: &Self::Carrier) -> bool { unimplemented!() }
    }

    pub struct FfiMeasurement {
        pub input_glue: FfiGlue<FfiDomain>,
        pub output_glue: FfiGlue<FfiDomain>,
        pub value: Box<Measurement<FfiDomain, FfiDomain>>,
    }

    impl FfiMeasurement {
        pub fn new_from_types<ID: 'static + Domain, OD: 'static + Domain>(value: Measurement<ID, OD>) -> *mut FfiMeasurement {
            let input_domain_glue = FfiGlue::<ID>::new_from_type();
            let input_domain_glue = unsafe { transmute(input_domain_glue) };
            let output_domain_glue = FfiGlue::<OD>::new_from_type();
            let output_domain_glue = unsafe { transmute(output_domain_glue) };
            Self::new(input_domain_glue, output_domain_glue, value)
        }

        pub fn new<ID: 'static + Domain, OD: 'static + Domain>(input_glue: FfiGlue<FfiDomain>, output_glue: FfiGlue<FfiDomain>, value: Measurement<ID, OD>) -> *mut FfiMeasurement {
            let value = ffi_utils::into_box(value);
            let ffi_measurement = FfiMeasurement { input_glue, output_glue, value };
            ffi_utils::into_raw(ffi_measurement)
        }
    }

    pub struct FfiTransformation {
        pub input_glue: FfiGlue<FfiDomain>,
        pub output_glue: FfiGlue<FfiDomain>,
        pub value: Box<Transformation<FfiDomain, FfiDomain>>,
    }

    impl FfiTransformation {
        pub fn new_from_types<ID: 'static + Domain, OD: 'static + Domain>(value: Transformation<ID, OD>) -> *mut FfiTransformation {
            let input_glue = FfiGlue::<ID>::new_from_type();
            let input_glue = unsafe { transmute(input_glue) };
            let output_glue = FfiGlue::<OD>::new_from_type();
            let output_glue = unsafe { transmute(output_glue) };
            Self::new(input_glue, output_glue, value)
        }

        pub fn new<ID: 'static + Domain, OD: 'static + Domain>(input_glue: FfiGlue<FfiDomain>, output_glue: FfiGlue<FfiDomain>, value: Transformation<ID, OD>) -> *mut FfiTransformation {
            let value = ffi_utils::into_box(value);
            let ffi_transformation = FfiTransformation { input_glue, output_glue, value };
            ffi_utils::into_raw(ffi_transformation)
        }
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__measurement_invoke(this: *const FfiMeasurement, arg: *const FfiObject) -> *mut FfiObject {
        let this = ffi_utils::as_ref(this);
        let arg = ffi_utils::as_ref(arg);
        assert_eq!(arg.type_, this.input_glue.carrier);
        let res_type = this.output_glue.carrier.clone();
        let res = this.value.function.eval_ffi(&arg.value);
        FfiObject::new_typed(res_type, res)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__measurement_free(this: *mut FfiMeasurement) {
        ffi_utils::into_owned(this);
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__transformation_invoke(this: *const FfiTransformation, arg: *const FfiObject) -> *mut FfiObject {
        let this = ffi_utils::as_ref(this);
        let arg = ffi_utils::as_ref(arg);
        assert_eq!(arg.type_, this.input_glue.carrier);
        let res_type = this.output_glue.carrier.clone();
        let res = this.value.function.eval_ffi(&arg.value);
        FfiObject::new_typed(res_type, res)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__transformation_free(this: *mut FfiTransformation) {
        ffi_utils::into_owned(this);
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_chain_mt(measurement1: *mut FfiMeasurement, transformation0: *mut FfiTransformation) -> *mut FfiMeasurement {
        let transformation0 = ffi_utils::as_ref(transformation0);
        let measurement1 = ffi_utils::as_ref(measurement1);
        assert_eq!(transformation0.output_glue.type_, measurement1.input_glue.type_);
        let input_glue = transformation0.input_glue.clone();
        let x_glue = transformation0.output_glue.clone();
        let output_glue = measurement1.output_glue.clone();
        let measurement = make_chain_mt_core(&measurement1.value, &transformation0.value, &input_glue, &x_glue, &output_glue);
        FfiMeasurement::new(input_glue, output_glue, measurement)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_chain_tt(transformation1: *mut FfiTransformation, transformation0: *mut FfiTransformation) -> *mut FfiTransformation {
        let transformation0 = ffi_utils::as_ref(transformation0);
        let transformation1 = ffi_utils::as_ref(transformation1);
        assert_eq!(transformation0.output_glue.type_, transformation1.input_glue.type_);
        let input_glue = transformation0.input_glue.clone();
        let x_glue = transformation0.output_glue.clone();
        let output_glue = transformation1.output_glue.clone();
        let transformation = make_chain_tt_core(&transformation1.value, &transformation0.value, &input_glue, &x_glue, &output_glue);
        FfiTransformation::new(input_glue, output_glue, transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_composition(measurement0: *mut FfiMeasurement, measurement1: *mut FfiMeasurement) -> *mut FfiMeasurement {
        let measurement0 = ffi_utils::as_ref(measurement0);
        let measurement1 = ffi_utils::as_ref(measurement1);
        assert_eq!(measurement0.input_glue.type_, measurement1.input_glue.type_);
        let input_glue = measurement0.input_glue.clone();
        let output_glue0 = measurement0.output_glue.clone();
        let output_glue1 = measurement1.output_glue.clone();
        // TODO: output_glue for composition.
        let output_glue_type = Type::new::<FfiDomain>();
        let output_glue_carrier = Type::new_box_pair(&output_glue0.carrier, &output_glue1.carrier);
        let output_glue_eq = Rc::new(|_d0: &Box<FfiDomain>, _d1: &Box<FfiDomain>| false);
        let output_glue_clone = Rc::new(|d: &Box<FfiDomain>| d.clone());
        let output_glue = FfiGlue::<FfiDomain>::new(output_glue_type, output_glue_carrier, output_glue_eq, output_glue_clone);
        let measurement = make_composition_core(&measurement0.value, &measurement1.value, &input_glue, &output_glue0, &output_glue1);
        FfiMeasurement::new(input_glue, output_glue, measurement)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__bootstrap() -> *const c_char {
        let spec =
r#"{
    "functions": [
        { "name": "measurement_invoke", "args": [ ["const void *", "this"], ["void *", "arg"] ], "ret": "void *" },
        { "name": "measurement_free", "args": [ ["void *", "this"] ] },
        { "name": "transformation_invoke", "args": [ ["const void *", "this"], ["void *", "arg"] ], "ret": "void *" },
        { "name": "transformation_free", "args": [ ["void *", "this"] ] },
        { "name": "make_chain_mt", "args": [ ["void *", "measurement"], ["void *", "transformation"] ], "ret": "void *" },
        { "name": "make_chain_tt", "args": [ ["void *", "transformation1"], ["void *", "transformation0"] ], "ret": "void *" },
        { "name": "make_composition", "args": [ ["void *", "transformation0"], ["void *", "transformation1"] ], "ret": "void *" }
    ]
}"#;
        ffi_utils::bootstrap(spec)
    }

}


// UNIT TESTS
#[cfg(test)]
mod tests {
    use crate::dom::AllDomain;

    use super::*;

    #[test]
    fn test_identity() {
        let identity = Transformation::new(AllDomain::<i32>::new(), AllDomain::<i32>::new(), |arg: &i32| arg.clone());
        let arg = 99;
        let ret = identity.function.eval(&arg);
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_make_chain_mt() {
        let transformation = Transformation::new(AllDomain::<u8>::new(), AllDomain::<i32>::new(), |a: &u8| (a + 1) as i32);
        let measurement = Measurement::new(AllDomain::<i32>::new(), AllDomain::<f64>::new(), |a: &i32| (a + 1) as f64);
        let chain = make_chain_mt(&measurement, &transformation);
        let arg = 99_u8;
        let ret = chain.function.eval(&arg);
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_chain_tt() {
        let transformation0 = Transformation::new(AllDomain::<u8>::new(), AllDomain::<i32>::new(), |a: &u8| (a + 1) as i32);
        let transformation1 = Transformation::new(AllDomain::<i32>::new(), AllDomain::<f64>::new(), |a: &i32| (a + 1) as f64);
        let chain = make_chain_tt(&transformation1, &transformation0);
        let arg = 99_u8;
        let ret = chain.function.eval(&arg);
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_composition() {
        let measurement0 = Measurement::new(AllDomain::<i32>::new(), AllDomain::<f32>::new(), |arg: &i32| (arg + 1) as f32);
        let measurement1 = Measurement::new(AllDomain::<i32>::new(), AllDomain::<f64>::new(), |arg: &i32| (arg - 1) as f64);
        let composition = make_composition(&measurement0, &measurement1);
        let arg = 99;
        let ret = composition.function.eval(&arg);
        assert_eq!(ret, (Box::new(100_f32), Box::new(98_f64)));
    }

}


// PLACEHOLDERS
mod dummy {
    use std::marker::PhantomData;

    use super::*;

    pub struct DummyDomain<T> {
        _marker: PhantomData<T>
    }
    impl<T> DummyDomain<T> {
        pub fn new() -> Self {
            DummyDomain { _marker: PhantomData }
        }
    }
    impl<T> Clone for DummyDomain<T> {
        fn clone(&self) -> Self { Self::new() }
    }
    impl<T: 'static> Domain for DummyDomain<T> {
        type Carrier=T;
        fn check_compatible(&self, _other: &Self) -> bool { true }
        fn check_valid(&self, _val: &Self::Carrier) -> bool { true }
    }

    pub struct DummyMetric<T> {
        _marker: PhantomData<T>
    }
    impl<T> DummyMetric<T> {
        pub fn new() -> Self {
            DummyMetric { _marker: PhantomData }
        }
    }
    impl<T> Clone for DummyMetric<T> {
        fn clone(&self) -> Self { Self::new() }
    }
    impl<T: 'static> Metric for DummyMetric<T> {
        type Distance=T;
    }

    pub struct DummyMeasure<T> {
        _marker: PhantomData<T>
    }
    impl<T> DummyMeasure<T> {
        pub fn new() -> Self {
            DummyMeasure { _marker: PhantomData }
        }
    }
    impl<T> Clone for DummyMeasure<T> {
        fn clone(&self) -> Self { Self::new() }
    }
    impl<T: 'static> Measure for DummyMeasure<T> {
        type Distance=T;
    }

    pub fn dummy_relation<I, O>() -> Relation<I, O> {
        Relation::new(|_i, _o| false)
    }
}
