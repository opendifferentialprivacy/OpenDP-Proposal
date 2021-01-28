use std::rc::Rc;

use crate::core::ffi::{DomainMeasureGlue, DomainMetricGlue};
use crate::dom::{BoxDomain, PairDomain};

// BUILDING BLOCKS
pub trait Domain: Clone + PartialEq {
    type Carrier;
    fn member(&self, val: &Self::Carrier) -> bool;
}

#[derive(Clone)]
pub struct Function<ID: Domain, OD: Domain> {
    function: Rc<dyn Fn(&ID::Carrier) -> Box<OD::Carrier>>
}

impl<ID: Domain, OD: Domain> Function<ID, OD> {
    pub fn new(function: impl Fn(&ID::Carrier) -> OD::Carrier + 'static) -> Self {
        let function = move |arg: &ID::Carrier| {
            let res = function(arg);
            Box::new(res)
        };
        let function = Rc::new(function);
        Function { function }
    }

    pub fn eval(&self, arg: &ID::Carrier) -> OD::Carrier {
        *(self.function)(arg)
    }

    pub fn eval_ffi(&self, arg: &ID::Carrier) -> Box<OD::Carrier> {
        (self.function)(arg)
    }
}

impl<ID: 'static + Domain, OD: 'static + Domain> Function<ID, OD> {
    pub fn make_chain<XD: 'static + Domain>(function1: &Function<XD, OD>, function0: &Function<ID, XD>) -> Function<ID, OD> {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        let function = move |arg: &ID::Carrier| {
            let res0 = function0(arg);
            function1(&res0)
        };
        let function = Rc::new(function);
        Function { function }
    }
}

impl<ID: 'static + Domain, ODA: 'static + Domain, ODB: 'static + Domain> Function<ID, PairDomain<BoxDomain<ODA>, BoxDomain<ODB>>> {
    pub fn make_composition(function0: &Function<ID, ODA>, function1: &Function<ID, ODB>) -> Self {
        let function0 = function0.function.clone();
        let function1 = function1.function.clone();
        let function = move |arg: & ID::Carrier| {
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
pub struct PrivacyRelation<IM: Metric, OM: Measure> {
    relation: Rc<dyn Fn(&IM::Distance, &OM::Distance) -> bool>
}
impl<IM: Metric, OM: Measure> PrivacyRelation<IM, OM> {
    pub fn new(relation: impl Fn(&IM::Distance, &OM::Distance) -> bool + 'static) -> Self {
        let relation = Rc::new(relation);
        PrivacyRelation { relation }
    }
    pub fn eval(&self, input_distance: &IM::Distance, output_distance: &OM::Distance) -> bool {
        (self.relation)(input_distance, output_distance)
    }
}

pub struct StabilityRelation<IM: Metric, OM: Metric> {
    relation: Rc<dyn Fn(&IM::Distance, &OM::Distance) -> bool>
}
impl<IM: Metric, OM: Metric> StabilityRelation<IM, OM> {
    pub fn new(relation: impl Fn(&IM::Distance, &OM::Distance) -> bool + 'static) -> Self {
        let relation = Rc::new(relation);
        StabilityRelation { relation }
    }
    pub fn eval(&self, input_distance: &IM::Distance, output_distance: &OM::Distance) -> bool {
        (self.relation)(input_distance, output_distance)
    }
}


// MEASUREMENTS & TRANSFORMATIONS
pub struct Measurement<ID: Domain, OD: Domain, IM: Metric, OM: Measure> {
    pub input_domain: Box<ID>,
    pub output_domain: Box<OD>,
    pub function: Function<ID, OD>,
    pub input_metric: Box<IM>,
    pub output_measure: Box<OM>,
    pub privacy_relation: PrivacyRelation<IM, OM>,
}

impl<ID: Domain, OD: Domain, IM: Metric, OM: Measure> Measurement<ID, OD, IM, OM> {
    pub fn new(
        input_domain: ID,
        output_domain: OD,
        function: impl Fn(&ID::Carrier) -> OD::Carrier + 'static,
        input_metric: IM,
        output_measure: OM,
        privacy_relation: impl Fn(&IM::Distance, &OM::Distance) -> bool + 'static,
    ) -> Self {
        let input_domain = Box::new(input_domain);
        let output_domain = Box::new(output_domain);
        let function = Function::new(function);
        let input_metric = Box::new(input_metric);
        let output_measure = Box::new(output_measure);
        let privacy_relation = PrivacyRelation::new(privacy_relation);
        Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
    }
}

pub struct Transformation<ID: Domain, OD: Domain, IM: Metric, OM: Metric> {
    pub input_domain: Box<ID>,
    pub output_domain: Box<OD>,
    pub function: Function<ID, OD>,
    pub input_metric: Box<IM>,
    pub output_metric: Box<OM>,
    pub stability_relation: StabilityRelation<IM, OM>,
}

impl<ID: Domain, OD: Domain, IM: Metric, OM: Metric> Transformation<ID, OD, IM, OM> {
    pub fn new(
        input_domain: ID,
        output_domain: OD,
        function: impl Fn(&ID::Carrier) -> OD::Carrier + 'static,
        input_metric: IM,
        output_metric: OM,
        stability_relation: impl Fn(&IM::Distance, &OM::Distance) -> bool + 'static,
    ) -> Self {
        let input_domain = Box::new(input_domain);
        let output_domain = Box::new(output_domain);
        let function = Function::new(function);
        let input_metric = Box::new(input_metric);
        let output_metric = Box::new(output_metric);
        let stability_relation = StabilityRelation::new(stability_relation);
        Transformation { input_domain, output_domain, function, input_metric, output_metric, stability_relation }
    }
}


// CHAINING & COMPOSITION
pub fn make_chain_mt<ID, XD, OD, IM, XM, OM>(measurement1: &Measurement<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>) -> Measurement<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, XM: 'static + Metric, OM: 'static + Measure {
    let input_glue = DomainMetricGlue::<ID, IM>::new_from_type();
    let x_glue = DomainMetricGlue::<XD, XM>::new_from_type();
    let output_glue = DomainMeasureGlue::<OD, OM>::new_from_type();
    make_chain_mt_core(measurement1, transformation0, &input_glue, &x_glue, &output_glue)
}

fn make_chain_mt_core<ID, XD, OD, IM, XM, OM>(measurement1: &Measurement<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>, input_glue: &DomainMetricGlue<ID, IM>, x_glue: &DomainMetricGlue<XD, XM>, output_glue: &DomainMeasureGlue<OD, OM>) -> Measurement<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, XM: 'static + Metric, OM: 'static + Measure {
    assert!((x_glue.domain_eq)(&transformation0.output_domain, &measurement1.input_domain));
    let input_domain = (input_glue.domain_clone)(&transformation0.input_domain);
    let output_domain = (output_glue.domain_clone)(&measurement1.output_domain);
    let function = Function::make_chain(&measurement1.function, &transformation0.function);
    let input_metric = (input_glue.metric_clone)(&transformation0.input_metric);
    let output_measure = (output_glue.measure_clone)(&measurement1.output_measure);
    // TODO: PrivacyRelation for make_chain_mt
    let privacy_relation = PrivacyRelation::new(|_i, _o| false);
    Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
}

pub fn make_chain_tt<ID, XD, OD, IM, XM, OM>(transformation1: &Transformation<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>) -> Transformation<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, XM: 'static + Metric, OM: 'static + Metric {
    let input_glue = DomainMetricGlue::<ID, IM>::new_from_type();
    let x_glue = DomainMetricGlue::<XD, XM>::new_from_type();
    let output_glue = DomainMetricGlue::<OD, OM>::new_from_type();
    make_chain_tt_core(transformation1, transformation0, &input_glue, &x_glue, &output_glue)
}

fn make_chain_tt_core<ID, XD, OD, IM, XM, OM>(transformation1: &Transformation<XD, OD, XM, OM>, transformation0: &Transformation<ID, XD, IM, XM>, input_glue: &DomainMetricGlue<ID, IM>, x_glue: &DomainMetricGlue<XD, XM>, output_glue: &DomainMetricGlue<OD, OM>) -> Transformation<ID, OD, IM, OM> where
    ID: 'static + Domain, XD: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, XM: 'static + Metric, OM: 'static + Metric {
    assert!((x_glue.domain_eq)(&transformation0.output_domain, &transformation1.input_domain));
    let input_domain = (input_glue.domain_clone)(&transformation0.input_domain);
    let output_domain = (output_glue.domain_clone)(&transformation1.output_domain);
    let function = Function::make_chain(&transformation1.function, &transformation0.function);
    let input_metric = (input_glue.metric_clone)(&transformation0.input_metric);
    let output_metric = (output_glue.metric_clone)(&transformation1.output_metric);
    // TODO: StabilityRelation for make_chain_tt
    let stability_relation = StabilityRelation::new(|_i, _o| false);
    Transformation { input_domain, output_domain, function, input_metric, output_metric, stability_relation }
}

pub fn make_composition<ID, OD0, OD1, IM, OM>(measurement0: &Measurement<ID, OD0, IM, OM>, measurement1: &Measurement<ID, OD1, IM, OM>) -> Measurement<ID, PairDomain<BoxDomain<OD0>, BoxDomain<OD1>>, IM, OM> where
    ID: 'static + Domain, OD0: 'static + Domain, OD1: 'static + Domain, IM: 'static + Metric, OM: 'static + Measure {
    let input_glue = DomainMetricGlue::<ID, IM>::new_from_type();
    let output_glue0 = DomainMeasureGlue::<OD0, OM>::new_from_type();
    let output_glue1 = DomainMeasureGlue::<OD1, OM>::new_from_type();
    make_composition_core(measurement0, measurement1, &input_glue, &output_glue0, &output_glue1)
}

fn make_composition_core<ID, OD0, OD1, IM, OM>(measurement0: &Measurement<ID, OD0, IM, OM>, measurement1: &Measurement<ID, OD1, IM, OM>, input_glue: &DomainMetricGlue<ID, IM>, output_glue0: &DomainMeasureGlue<OD0, OM>, output_glue1: &DomainMeasureGlue<OD1, OM>) -> Measurement<ID, PairDomain<BoxDomain<OD0>, BoxDomain<OD1>>, IM, OM> where
    ID: 'static + Domain, OD0: 'static + Domain, OD1: 'static + Domain, IM: 'static + Metric, OM: 'static + Measure {
    assert!((input_glue.domain_eq)(&measurement0.input_domain, &measurement1.input_domain));
    let input_domain = (input_glue.domain_clone)(&measurement0.input_domain);
    let output_domain0 = (output_glue0.domain_clone)(&measurement0.output_domain);
    let output_domain0 = BoxDomain::new(output_domain0);
    let output_domain1 = (output_glue1.domain_clone)(&measurement1.output_domain);
    let output_domain1 = BoxDomain::new(output_domain1);
    let output_domain = PairDomain::new(output_domain0, output_domain1);
    let output_domain = Box::new(output_domain);
    let function = Function::make_composition(&measurement0.function, &measurement1.function);
    // TODO: Figure out input_metric for composition.
    let input_metric = (input_glue.metric_clone)(&measurement0.input_metric);
    // TODO: Figure out output_measure for composition.
    let output_measure = (output_glue0.measure_clone)(&measurement0.output_measure);
    // TODO: PrivacyRelation for make_composition
    let privacy_relation = PrivacyRelation::new(|_i, _o| false);
    Measurement { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
}


// FFI BINDINGS
pub(crate) mod ffi {
    use std::intrinsics::transmute;
    use std::os::raw::c_char;

    use crate::ffi_utils;
    use crate::ffi_utils::Type;

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
    pub struct DomainMetricGlue<D: Domain, M: Metric> {
        pub domain_type: Type,
        pub domain_carrier: Type,
        pub domain_eq: Rc<dyn Fn(&Box<D>, &Box<D>) -> bool>,
        pub domain_clone: Rc<dyn Fn(&Box<D>) -> Box<D>>,
        pub metric_clone: Rc<dyn Fn(&Box<M>) -> Box<M>>,
    }
    impl<D: 'static + Domain, M: 'static + Metric> DomainMetricGlue<D, M> {
        pub fn new_from_type() -> Self {
            let domain_type = Type::new::<D>();
            let domain_carrier = Type::new::<D::Carrier>();
            let domain_eq = |d0: &Box<D>, d1: &Box<D>| d0 == d1;
            let domain_eq = Rc::new(domain_eq);
            let domain_clone = |d: &Box<D>| d.clone();
            let domain_clone = Rc::new(domain_clone);
            let metric_clone = |m: &Box<M>| m.clone();
            let metric_clone = Rc::new(metric_clone);
            Self::new(domain_type, domain_carrier, domain_eq, domain_clone, metric_clone)
        }
        pub fn new(domain_type: Type, domain_carrier: Type, domain_eq: Rc<dyn Fn(&Box<D>, &Box<D>) -> bool>, domain_clone: Rc<dyn Fn(&Box<D>) -> Box<D>>, metric_clone: Rc<dyn Fn(&Box<M>) -> Box<M>>) -> Self {
            DomainMetricGlue { domain_type, domain_carrier, domain_eq, domain_clone, metric_clone }
        }
    }

    #[derive(Clone)]
    pub struct DomainMeasureGlue<D: Domain, M: Measure> {
        pub domain_type: Type,
        pub domain_carrier: Type,
        pub domain_eq: Rc<dyn Fn(&Box<D>, &Box<D>) -> bool>,
        pub domain_clone: Rc<dyn Fn(&Box<D>) -> Box<D>>,
        pub measure_clone: Rc<dyn Fn(&Box<M>) -> Box<M>>,
    }
    impl<D: 'static + Domain, M: 'static + Measure> DomainMeasureGlue<D, M> {
        pub fn new_from_type() -> Self {
            let domain_type = Type::new::<D>();
            let domain_carrier = Type::new::<D::Carrier>();
            let domain_eq = |d0: &Box<D>, d1: &Box<D>| d0 == d1;
            let domain_eq = Rc::new(domain_eq);
            let domain_clone = |d: &Box<D>| d.clone();
            let domain_clone = Rc::new(domain_clone);
            let measure_clone = |m: &Box<M>| m.clone();
            let measure_clone = Rc::new(measure_clone);
            Self::new(domain_type, domain_carrier, domain_eq, domain_clone, measure_clone)
        }
        pub fn new(domain_type: Type, domain_carrier: Type, domain_eq: Rc<dyn Fn(&Box<D>, &Box<D>) -> bool>, domain_clone: Rc<dyn Fn(&Box<D>) -> Box<D>>, measure_clone: Rc<dyn Fn(&Box<M>) -> Box<M>>) -> Self {
            DomainMeasureGlue { domain_type, domain_carrier, domain_eq, domain_clone, measure_clone }
        }
    }

    #[derive(Clone, PartialEq)]
    pub struct FfiDomain;
    impl Domain for FfiDomain {
        type Carrier = ();
        fn member(&self, _val: &Self::Carrier) -> bool { unimplemented!() }
    }

    #[derive(Clone)]
    pub struct FfiMetric;
    impl Metric for FfiMetric {
        type Distance = ();
    }

    #[derive(Clone)]
    pub struct FfiMeasure;
    impl Measure for FfiMeasure {
        type Distance = ();
    }

    pub struct FfiMeasurement {
        pub input_glue: DomainMetricGlue<FfiDomain, FfiMetric>,
        pub output_glue: DomainMeasureGlue<FfiDomain, FfiMeasure>,
        pub value: Box<Measurement<FfiDomain, FfiDomain, FfiMetric, FfiMeasure>>,
    }

    impl FfiMeasurement {
        pub fn new_from_types<ID: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, OM: 'static + Measure>(value: Measurement<ID, OD, IM, OM>) -> *mut FfiMeasurement {
            let input_glue = DomainMetricGlue::<ID, IM>::new_from_type();
            let input_glue = unsafe { transmute(input_glue) };
            let output_glue = DomainMeasureGlue::<OD, OM>::new_from_type();
            let output_glue = unsafe { transmute(output_glue) };
            Self::new(input_glue, output_glue, value)
        }

        pub fn new<ID: 'static + Domain, OD: 'static + Domain, IM: Metric, OM: Measure>(input_glue: DomainMetricGlue<FfiDomain, FfiMetric>, output_glue: DomainMeasureGlue<FfiDomain, FfiMeasure>, value: Measurement<ID, OD, IM, OM>) -> *mut FfiMeasurement {
            let value = ffi_utils::into_box(value);
            let ffi_measurement = FfiMeasurement { input_glue, output_glue, value };
            ffi_utils::into_raw(ffi_measurement)
        }
    }

    pub struct FfiTransformation {
        pub input_glue: DomainMetricGlue<FfiDomain, FfiMetric>,
        pub output_glue: DomainMetricGlue<FfiDomain, FfiMetric>,
        pub value: Box<Transformation<FfiDomain, FfiDomain, FfiMetric, FfiMetric>>,
    }

    impl FfiTransformation {
        pub fn new_from_types<ID: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, OM: 'static + Metric>(value: Transformation<ID, OD, IM, OM>) -> *mut FfiTransformation {
            let input_glue = DomainMetricGlue::<ID, IM>::new_from_type();
            let input_glue = unsafe { transmute(input_glue) };
            let output_glue = DomainMetricGlue::<OD, OM>::new_from_type();
            let output_glue = unsafe { transmute(output_glue) };
            Self::new(input_glue, output_glue, value)
        }

        pub fn new<ID: 'static + Domain, OD: 'static + Domain, IM: Metric, OM: Metric>(input_glue: DomainMetricGlue<FfiDomain, FfiMetric>, output_glue: DomainMetricGlue<FfiDomain, FfiMetric>, value: Transformation<ID, OD, IM, OM>) -> *mut FfiTransformation {
            let value = ffi_utils::into_box(value);
            let ffi_transformation = FfiTransformation { input_glue, output_glue, value };
            ffi_utils::into_raw(ffi_transformation)
        }
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__measurement_invoke(this: *const FfiMeasurement, arg: *const FfiObject) -> *mut FfiObject {
        let this = ffi_utils::as_ref(this);
        let arg = ffi_utils::as_ref(arg);
        assert_eq!(arg.type_, this.input_glue.domain_carrier);
        let res_type = this.output_glue.domain_carrier.clone();
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
        assert_eq!(arg.type_, this.input_glue.domain_carrier);
        let res_type = this.output_glue.domain_carrier.clone();
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
        assert_eq!(transformation0.output_glue.domain_type, measurement1.input_glue.domain_type);
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
        assert_eq!(transformation0.output_glue.domain_type, transformation1.input_glue.domain_type);
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
        assert_eq!(measurement0.input_glue.domain_type, measurement1.input_glue.domain_type);
        let input_glue = measurement0.input_glue.clone();
        let output_glue0 = measurement0.output_glue.clone();
        let output_glue1 = measurement1.output_glue.clone();
        // TODO: output_glue for composition.
        let output_glue_domain_type = Type::new::<FfiDomain>();
        let output_glue_domain_carrier = Type::new_box_pair(&output_glue0.domain_carrier, &output_glue1.domain_carrier);
        let output_glue_domain_eq = Rc::new(|_d0: &Box<FfiDomain>, _d1: &Box<FfiDomain>| false);
        let output_glue_domain_clone = Rc::new(|d: &Box<FfiDomain>| d.clone());
        let output_glue_measure_clone = Rc::new(|d: &Box<FfiMeasure>| d.clone());
        let output_glue = DomainMeasureGlue::<FfiDomain, FfiMeasure>::new(output_glue_domain_type, output_glue_domain_carrier, output_glue_domain_eq, output_glue_domain_clone, output_glue_measure_clone);
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
    use crate::dis::{L1Sensitivity, MaxDivergence};
    use crate::dom::AllDomain;

    use super::*;

    #[test]
    fn test_identity() {
        let input_domain = AllDomain::<i32>::new();
        let output_domain = AllDomain::<i32>::new();
        let function = |arg: &i32| arg.clone();
        let input_metric = L1Sensitivity::<i32>::new();
        let output_metric = L1Sensitivity::<i32>::new();
        let stability_relation = |_d_in: &i32, _d_out: &i32| true;
        let identity = Transformation::new(input_domain, output_domain, function, input_metric, output_metric, stability_relation);
        let arg = 99;
        let ret = identity.function.eval(&arg);
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_make_chain_mt() {
        let input_domain0 = AllDomain::<u8>::new();
        let output_domain0 = AllDomain::<i32>::new();
        let function0 = |a: &u8| (a + 1) as i32;
        let input_metric0 = L1Sensitivity::<i32>::new();
        let output_metric0 = L1Sensitivity::<i32>::new();
        let stability_relation0 = |_d_in: &i32, _d_out: &i32| true;
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = |a: &i32| (a + 1) as f64;
        let input_metric1 = L1Sensitivity::<i32>::new();
        let output_measure1 = MaxDivergence::new();
        let privacy_relation1 = |_d_in: &i32, _d_out: &f64| true;
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let chain = make_chain_mt(&measurement1, &transformation0);
        let arg = 99_u8;
        let ret = chain.function.eval(&arg);
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_chain_tt() {
        let input_domain0 = AllDomain::<u8>::new();
        let output_domain0 = AllDomain::<i32>::new();
        let function0 = |a: &u8| (a + 1) as i32;
        let input_metric0 = L1Sensitivity::<i32>::new();
        let output_metric0 = L1Sensitivity::<i32>::new();
        let stability_relation0 = |_d_in: &i32, _d_out: &i32| true;
        let transformation0 = Transformation::new(input_domain0, output_domain0, function0, input_metric0, output_metric0, stability_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = |a: &i32| (a + 1) as f64;
        let input_metric1 = L1Sensitivity::<i32>::new();
        let output_metric1 = L1Sensitivity::<i32>::new();
        let stability_relation1 = |_d_in: &i32, _d_out: &i32| true;
        let transformation1 = Transformation::new(input_domain1, output_domain1, function1, input_metric1, output_metric1, stability_relation1);
        let chain = make_chain_tt(&transformation1, &transformation0);
        let arg = 99_u8;
        let ret = chain.function.eval(&arg);
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_composition() {
        let input_domain0 = AllDomain::<i32>::new();
        let output_domain0 = AllDomain::<f32>::new();
        let function0 = |arg: &i32| (arg + 1) as f32;
        let input_metric0 = L1Sensitivity::<i32>::new();
        let output_measure0 = MaxDivergence::new();
        let privacy_relation0 = |_d_in: &i32, _d_out: &f64| true;
        let measurement0 = Measurement::new(input_domain0, output_domain0, function0, input_metric0, output_measure0, privacy_relation0);
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = |arg: &i32| (arg - 1) as f64;
        let input_metric1 = L1Sensitivity::<i32>::new();
        let output_measure1 = MaxDivergence::new();
        let privacy_relation1 = |_d_in: &i32, _d_out: &f64| true;
        let measurement1 = Measurement::new(input_domain1, output_domain1, function1, input_metric1, output_measure1, privacy_relation1);
        let composition = make_composition(&measurement0, &measurement1);
        let arg = 99;
        let ret = composition.function.eval(&arg);
        assert_eq!(ret, (Box::new(100_f32), Box::new(98_f64)));
    }

}
