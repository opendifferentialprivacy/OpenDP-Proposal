use std::rc::Rc;

use crate::data::TraitObject;
use crate::dom::AllDomain;
use crate::ffi_utils;
use crate::core::dummy::{DummyMetric, DummyMeasure, dummy_relation, DummyDomain};


// BUILDING BLOCKS
pub trait Domain: TraitObject {
    type Carrier;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>>;
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool;
    fn check_valid(&self, val: &Self::Carrier) -> bool;
}

pub trait DomainAlt: Clone {
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
pub struct Measurement<I, O, IM: Metric=DummyMetric<()>, OM: Measure=DummyMeasure<()>> {
    pub input_domain: Box<dyn Domain<Carrier=I>>,
    pub input_metric: IM,
    pub output_domain: Box<dyn Domain<Carrier=O>>,
    pub output_measure: OM,
    pub privacy_relation: Relation<IM::Distance, OM::Distance>,
    function: Rc<dyn Fn(*const I) -> Box<O>>,
}

impl<I, O> Measurement<I, O> {
    pub fn new(input_domain: impl Domain<Carrier=I> + 'static, output_domain: impl Domain<Carrier=O> + 'static, function: impl Fn(&I) -> O + 'static) -> Self {
        let input_metric = dummy::DummyMetric::<()>::new();
        let output_measure = dummy::DummyMeasure::<()>::new();
        let privacy_relation = dummy::dummy_relation();
        Self::new_all(input_domain, input_metric, output_domain, output_measure, privacy_relation, function)
    }
}

impl<I, O, IM: Metric, OM: Measure> Measurement<I, O, IM, OM> {
    pub fn new_all(
        input_domain: impl Domain<Carrier=I> + 'static,
        input_metric: IM,
        output_domain: impl Domain<Carrier=O> + 'static,
        output_measure: OM,
        privacy_relation: Relation<IM::Distance, OM::Distance>,
        function: impl Fn(&I) -> O + 'static
    ) -> Measurement<I, O, IM, OM> {
        let function = move |arg: *const I| -> Box<O> {
            let arg = ffi_utils::as_ref(arg);
            let res = function(arg);
            Box::new(res)
        };
        Measurement {
            input_domain: Box::new(input_domain),
            input_metric: input_metric,
            output_domain: Box::new(output_domain),
            output_measure: output_measure,
            privacy_relation: privacy_relation,
            function: Rc::new(function)
        }
    }

    pub fn invoke(&self, arg: &I) -> O {
        let arg = arg as *const I;
        let res = (self.function)(arg);
        *res
    }

    pub fn invoke_ffi(&self, arg: &Box<I>) -> Box<O> {
        let arg = arg.as_ref() as *const I;
        (self.function)(arg)
    }
}

pub struct MeasurementAlt<ID: DomainAlt, OD: DomainAlt, IM: Metric=DummyMetric<()>, OM: Measure=DummyMeasure<()>> {
    pub input_domain: Box<ID>,
    pub output_domain: Box<OD>,
    pub function: Function<ID::Carrier, OD::Carrier>,
    pub input_metric: IM,
    pub output_measure: OM,
    pub privacy_relation: Relation<IM::Distance, OM::Distance>,
}

impl<ID: DomainAlt, OD: DomainAlt> MeasurementAlt<ID, OD> {
    pub fn new(input_domain: ID, output_domain: OD, function: impl Fn(&ID::Carrier) -> OD::Carrier + 'static) -> Self {
        let function = Function::new(function);
        let input_metric = dummy::DummyMetric::new();
        let output_measure = dummy::DummyMeasure::new();
        let privacy_relation = dummy::dummy_relation();
        Self::new_all(input_domain, output_domain, function, input_metric, output_measure, privacy_relation)
    }
}

impl<ID: DomainAlt, OD: DomainAlt, IM: Metric, OM: Measure> MeasurementAlt<ID, OD, IM, OM> {
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
        MeasurementAlt { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
    }
}

pub struct Transformation<I, O, IM: Metric=DummyMetric<()>, OM: Metric=DummyMetric<()>> {
    pub input_domain: Box<dyn Domain<Carrier=I>>,
    pub input_metric: IM,
    pub output_domain: Box<dyn Domain<Carrier=O>>,
    pub output_metric: OM,
    pub stability_relation: Relation<IM::Distance, OM::Distance>,
    function: Rc<dyn Fn(*const I) -> Box<O>>,
}

impl<I, O> Transformation<I, O> {
    pub fn new(input_domain: impl Domain<Carrier=I> + 'static, output_domain: impl Domain<Carrier=O> + 'static, function: impl Fn(&I) -> O + 'static) -> Transformation<I, O> {
        let input_metric = dummy::DummyMetric::<()>::new();
        let output_metric = dummy::DummyMetric::<()>::new();
        let stability_relation = dummy::dummy_relation();
        Self::new_all(input_domain, input_metric, output_domain, output_metric, stability_relation, function)
    }
}

impl<I, O, IM: Metric, OM: Metric> Transformation<I, O, IM, OM> {
    pub fn new_all(
        input_domain: impl Domain<Carrier=I> + 'static,
        input_metric: IM,
        output_domain: impl Domain<Carrier=O> + 'static,
        output_metric: OM,
        stability_relation: Relation<IM::Distance, OM::Distance>,
        function: impl Fn(&I) -> O + 'static
    ) -> Self {
        let function = move |arg: *const I| -> Box<O> {
            let arg = ffi_utils::as_ref(arg);
            let res = function(arg);
            Box::new(res)
        };
        Transformation {
            input_domain: Box::new(input_domain),
            input_metric: input_metric,
            output_domain: Box::new(output_domain),
            output_metric: output_metric,
            stability_relation: stability_relation,
            function: Rc::new(function)
        }
    }

    pub fn invoke(&self, arg: &I) -> O {
        let arg = arg as *const I;
        let res = (self.function)(arg);
        *res
    }

    pub fn invoke_ffi(&self, arg: &Box<I>) -> Box<O> {
        let arg = arg.as_ref() as *const I;
        (self.function)(arg)
    }
}

pub struct TransformationAlt<ID: DomainAlt, OD: DomainAlt, IM: Metric=DummyMetric<()>, OM: Metric=DummyMetric<()>> {
    pub input_domain: Box<ID>,
    pub output_domain: Box<OD>,
    pub function: Function<ID::Carrier, OD::Carrier>,
    pub input_metric: IM,
    pub output_metric: OM,
    pub stability_relation: Relation<IM::Distance, OM::Distance>,
}

impl<ID: DomainAlt, OD: DomainAlt> TransformationAlt<ID, OD> {
    pub fn new(input_domain: ID, output_domain: OD, function: impl Fn(&ID::Carrier) -> OD::Carrier + 'static) -> Self {
        let function = Function::new(function);
        let input_metric = dummy::DummyMetric::new();
        let output_metric = dummy::DummyMetric::new();
        let stability_relation = dummy::dummy_relation();
        Self::new_all(input_domain, output_domain, function, input_metric, output_metric, stability_relation)
    }
}

impl<ID: DomainAlt, OD: DomainAlt, IM: Metric, OM: Metric> TransformationAlt<ID, OD, IM, OM> {
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
        TransformationAlt { input_domain, output_domain, function, input_metric, output_metric, stability_relation }
    }
}


// CHAINING & COMPOSITION
pub fn make_chain_mt<I: 'static, X: 'static, O: 'static>(measurement: &Measurement<X, O>, transformation: &Transformation<I, X>) -> Measurement<I, O> {
    assert!(transformation.output_domain.check_compatible(measurement.input_domain.as_ref()));
    let input_domain = transformation.input_domain.box_clone();
    let input_metric = transformation.input_metric.clone();
    let output_domain = measurement.output_domain.box_clone();
    let output_measure = measurement.output_measure.clone();
    let function0 = transformation.function.clone();
    let function1 = measurement.function.clone();
    let function = move |arg: *const I| -> Box<O> {
        let res0 = function0(arg);
        function1(&*res0)
    };
    let function = Box::new(function);
    Measurement {
        input_domain: input_domain,
        input_metric: input_metric,
        output_domain: output_domain,
        output_measure: output_measure,
        privacy_relation: dummy::dummy_relation(),
        function: Rc::new(function)
    }
}

pub fn make_chain_mt_alt<ID, XD, OD, IM, XM, OM>(measurement: &MeasurementAlt<XD, OD, XM, OM>, transformation0: &TransformationAlt<ID, XD, IM, XM>) -> MeasurementAlt<ID, OD, IM, OM> where
    ID: 'static + DomainAlt, XD: 'static + DomainAlt, OD: 'static + DomainAlt, IM: Metric, XM: Metric, OM: Measure {
    assert!(transformation0.output_domain.check_compatible(&measurement.input_domain));
    let input_domain = transformation0.input_domain.clone();
    let output_domain = measurement.output_domain.clone();
    let function = Function::make_chain(&measurement.function, &transformation0.function);
    let input_metric = transformation0.input_metric.clone();
    let output_measure = measurement.output_measure.clone();
    let privacy_relation = dummy_relation();
    MeasurementAlt { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
}

pub fn make_chain_tt<I: 'static, X: 'static, O: 'static>(transformation1: &Transformation<X, O>, transformation0: &Transformation<I, X>) -> Transformation<I, O> {
    assert!(transformation0.output_domain.check_compatible(transformation1.input_domain.as_ref()));
    let input_domain = transformation0.input_domain.box_clone();
    let input_metric = transformation0.input_metric.clone();
    let output_domain = transformation1.output_domain.box_clone();
    let output_metric = transformation1.output_metric.clone();
    let function0 = transformation0.function.clone();
    let function1 = transformation1.function.clone();
    let function = move |arg: *const I| -> Box<O> {
        let res0 = function0(arg);
        function1(&*res0)
    };
    let function = Box::new(function);
    Transformation {
        input_domain: input_domain,
        input_metric: input_metric,
        output_domain: output_domain,
        output_metric: output_metric,
        stability_relation: dummy::dummy_relation(),
        function: Rc::new(function)
    }
}

pub fn make_chain_tt_alt<ID, XD, OD, IM, XM, OM>(transformation1: &TransformationAlt<XD, OD, XM, OM>, transformation0: &TransformationAlt<ID, XD, IM, XM>) -> TransformationAlt<ID, OD, IM, OM> where
    ID: 'static + DomainAlt, XD: 'static + DomainAlt, OD: 'static + DomainAlt, IM: Metric, XM: Metric, OM: Metric {
    assert!(transformation0.output_domain.check_compatible(&transformation1.input_domain));
    let input_domain = transformation0.input_domain.clone();
    let output_domain = transformation1.output_domain.clone();
    let function = Function::make_chain(&transformation1.function, &transformation0.function);
    let input_metric = transformation0.input_metric.clone();
    let output_metric = transformation1.output_metric.clone();
    let stability_relation = dummy_relation();
    TransformationAlt { input_domain, output_domain, function, input_metric, output_metric, stability_relation }
}

pub fn make_composition<I: 'static, OA: 'static, OB: 'static>(measurement0: &Measurement<I, OA>, measurement1: &Measurement<I, OB>) -> Measurement<I, (Box<OA>, Box<OB>)> {
    assert!(measurement0.input_domain.check_compatible(measurement1.input_domain.as_ref()));
    let input_domain = measurement0.input_domain.box_clone();
    // TODO: Figure out input_metric for composition.
    let input_metric = measurement0.input_metric.clone();
    let _output_domain0 = measurement0.output_domain.box_clone();
    let _output_domain1 = measurement1.output_domain.box_clone();
    // TODO: Figure out output_domain for composition.
    let output_domain = Box::new(AllDomain::<(Box<OA>, Box<OB>)>::new());
    // TODO: Figure out output_measure for composition.
    let output_measure = measurement0.output_measure.clone();
    let function0 = measurement0.function.clone();
    let function1 = measurement1.function.clone();
    let function = move |arg: *const I| -> Box<(Box<OA>, Box<OB>)> {
        let res0 = function0(arg);
        let res1 = function1(arg);
        Box::new((res0, res1))
    };
    Measurement {
        input_domain: input_domain,
        input_metric: input_metric,
        output_domain: output_domain,
        output_measure: output_measure,
        privacy_relation: dummy::dummy_relation(),
        function: Rc::new(function)
    }
}

pub fn make_composition_alt<ID, OD0, OD1, IM, OM>(measurement0: &MeasurementAlt<ID, OD0, IM, OM>, measurement1: &MeasurementAlt<ID, OD1, IM, OM>) -> MeasurementAlt<ID, DummyDomain<(Box<OD0::Carrier>, Box<OD1::Carrier>)>, IM, OM> where
    ID: 'static + DomainAlt, OD0: 'static + DomainAlt, OD1: 'static + DomainAlt, IM: Metric, OM: Measure {
    // TODO: Equality check for input domains
    assert!(measurement0.input_domain.check_compatible(&measurement1.input_domain));
    let input_domain = measurement0.input_domain.clone();
    let _output_domain0 = measurement0.output_domain.clone();
    let _output_domain1 = measurement1.output_domain.clone();
    // TODO: Figure out output_domain for composition.
    let output_domain = Box::new(dummy::DummyDomain::new());
    let function = Function::make_composition(&measurement0.function, &measurement1.function);
    // TODO: Figure out input_metric for composition.
    let input_metric = measurement0.input_metric.clone();
    // TODO: Figure out output_measure for composition.
    let output_measure = measurement0.output_measure.clone();
    let privacy_relation = dummy_relation();
    MeasurementAlt { input_domain, output_domain, function, input_metric, output_measure, privacy_relation }
}


// FFI BINDINGS
pub(crate) mod ffi {
    use std::os::raw::c_char;

    use crate::ffi_utils;
    use crate::mono::Type;

    use super::*;
    use crate::dom::AllDomainAlt;

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

    pub struct FfiMeasurement {
        pub input_type: Type,
        pub output_type: Type,
        pub value: Box<Measurement<(), ()>>,
    }

    impl FfiMeasurement {
        pub fn new_typed<I, O>(input_type: Type, output_type: Type, value: Measurement<I, O>) -> *mut FfiMeasurement {
            let value = ffi_utils::into_box(value);
            let measurement = FfiMeasurement { input_type, output_type, value };
            ffi_utils::into_raw(measurement)
        }

        pub fn new<I: 'static, O: 'static>(value: Measurement<I, O>) -> *mut FfiMeasurement {
            let input_type = Type::new::<I>();
            let output_type = Type::new::<O>();
            Self::new_typed(input_type, output_type, value)
        }

        // pub fn as_ref<I, O>(&self) -> &Measurement<I, O> {
        //     // TODO: Check types.
        //     let value = self.value.as_ref() as *const Measurement<(), ()> as *const Measurement<I, O>;
        //     let value = unsafe { value.as_ref() };
        //     value.unwrap()
        // }
    }

    pub struct FfiMeasurementAlt {
        pub input_domain_carrier: Type,
        pub output_domain_carrier: Type,
        pub value: Box<MeasurementAlt<AllDomainAlt<()>, AllDomainAlt<()>>>,
    }

    impl FfiMeasurementAlt {
        pub fn new_typed<ID: DomainAlt, OD: DomainAlt>(input_domain_carrier: Type, output_domain_carrier: Type, value: MeasurementAlt<ID, OD>) -> *mut FfiMeasurementAlt {
            let value = ffi_utils::into_box(value);
            let measurement = FfiMeasurementAlt { input_domain_carrier, output_domain_carrier, value };
            ffi_utils::into_raw(measurement)
        }

        pub fn new<ID: 'static + DomainAlt, OD: 'static + DomainAlt>(value: MeasurementAlt<ID, OD>) -> *mut FfiMeasurementAlt {
            let input_domain_carrier = Type::new::<ID::Carrier>();
            let output_domain_carrier = Type::new::<OD::Carrier>();
            Self::new_typed(input_domain_carrier, output_domain_carrier, value)
        }

        // pub fn as_ref<ID: 'static + DomainAlt, OD: 'static + DomainAlt>(&self) -> &MeasurementAlt<ID, OD> {
        //     // TODO: Check types.
        //     let value = self.value.as_ref() as *const MeasurementAlt<AllDomainAlt<()>, AllDomainAlt<()>> as *const MeasurementAlt<ID, OD>;
        //     let value = unsafe { value.as_ref() };
        //     value.unwrap()
        // }
    }

    pub struct FfiTransformation {
        pub input_type: Type,
        pub output_type: Type,
        pub value: Box<Transformation<(), ()>>,
    }

    impl FfiTransformation {
        pub fn new_typed<I, O>(input_type: Type, output_type: Type, value: Transformation<I, O>) -> *mut FfiTransformation {
            let value = ffi_utils::into_box(value);
            let transformation = FfiTransformation { input_type, output_type, value };
            ffi_utils::into_raw(transformation)
        }

        pub fn new<I: 'static, O: 'static>(value: Transformation<I, O>) -> *mut FfiTransformation {
            let input_type = Type::new::<I>();
            let output_type = Type::new::<O>();
            Self::new_typed(input_type, output_type, value)
        }

        pub fn as_ref<I, O>(&self) -> &Transformation<I, O> {
            // TODO: Check types.
            let value = self.value.as_ref() as *const Transformation<(), ()> as *const Transformation<I, O>;
            let value = unsafe { value.as_ref() };
            value.unwrap()
        }
    }

    pub struct FfiTransformationAlt {
        pub input_domain_carrier: Type,
        pub output_domain_carrier: Type,
        pub value: Box<TransformationAlt<AllDomainAlt<()>, AllDomainAlt<()>>>,
    }

    impl FfiTransformationAlt {
        pub fn new_typed<ID: DomainAlt, OD: DomainAlt>(input_domain_carrier: Type, output_domain_carrier: Type, value: TransformationAlt<ID, OD>) -> *mut FfiTransformationAlt {
            let value = ffi_utils::into_box(value);
            let transformation = FfiTransformationAlt { input_domain_carrier, output_domain_carrier, value };
            ffi_utils::into_raw(transformation)
        }

        pub fn new<ID: 'static + DomainAlt, OD: 'static + DomainAlt>(value: TransformationAlt<ID, OD>) -> *mut FfiTransformationAlt {
            let input_domain_carrier = Type::new::<ID::Carrier>();
            let output_domain_carrier = Type::new::<OD::Carrier>();
            Self::new_typed(input_domain_carrier, output_domain_carrier, value)
        }

        // pub fn as_ref<ID: DomainAlt, OD: DomainAlt>(&self) -> &TransformationAlt<ID, OD> {
        //     // TODO: Check types.
        //     let value = self.value.as_ref() as *const TransformationAlt<AllDomainAlt<()>, AllDomainAlt<()>> as *const TransformationAlt<ID, OD>;
        //     let value = unsafe { value.as_ref() };
        //     value.unwrap()
        // }
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__measurement_invoke(this: *const FfiMeasurement, arg: *const FfiObject) -> *mut FfiObject {
        let this = ffi_utils::as_ref(this);
        let arg = ffi_utils::as_ref(arg);
        assert_eq!(arg.type_, this.input_type);
        let res_type = this.output_type.clone();
        let res = this.value.invoke_ffi(&arg.value);
        FfiObject::new_typed(res_type, res)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__measurement_invoke_alt(this: *const FfiMeasurementAlt, arg: *const FfiObject) -> *mut FfiObject {
        let this = ffi_utils::as_ref(this);
        let arg = ffi_utils::as_ref(arg);
        assert_eq!(arg.type_, this.input_domain_carrier);
        let res_type = this.output_domain_carrier.clone();
        let res = this.value.function.eval_ffi(&arg.value);
        FfiObject::new_typed(res_type, res)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__measurement_free(this: *mut FfiMeasurement) {
        ffi_utils::into_owned(this);
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__measurement_free_alt(this: *mut FfiMeasurementAlt) {
        ffi_utils::into_owned(this);
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__transformation_invoke(this: *const FfiTransformation, arg: *const FfiObject) -> *mut FfiObject {
        let this = ffi_utils::as_ref(this);
        let arg = ffi_utils::as_ref(arg);
        assert_eq!(arg.type_, this.input_type);
        let res_type = this.output_type.clone();
        let res = this.value.invoke_ffi(&arg.value);
        FfiObject::new_typed(res_type, res)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__transformation_invoke_alt(this: *const FfiTransformationAlt, arg: *const FfiObject) -> *mut FfiObject {
        let this = ffi_utils::as_ref(this);
        let arg = ffi_utils::as_ref(arg);
        assert_eq!(arg.type_, this.input_domain_carrier);
        let res_type = this.output_domain_carrier.clone();
        let res = this.value.function.eval_ffi(&arg.value);
        FfiObject::new_typed(res_type, res)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__transformation_free(this: *mut FfiTransformation) {
        ffi_utils::into_owned(this);
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__transformation_free_alt(this: *mut FfiTransformationAlt) {
        ffi_utils::into_owned(this);
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_chain_mt(measurement1: *mut FfiMeasurement, transformation0: *mut FfiTransformation) -> *mut FfiMeasurement {
        let transformation0 = ffi_utils::as_ref(transformation0);
        let measurement1 = ffi_utils::as_ref(measurement1);
        assert_eq!(transformation0.output_type, measurement1.input_type);
        let input_type = transformation0.input_type.clone();
        let output_type = measurement1.output_type.clone();
        let measurement = super::make_chain_mt(&measurement1.value, &transformation0.value);
        FfiMeasurement::new_typed(input_type, output_type, measurement)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_chain_mt_alt(measurement1: *mut FfiMeasurementAlt, transformation0: *mut FfiTransformationAlt) -> *mut FfiMeasurementAlt {
        let transformation0 = ffi_utils::as_ref(transformation0);
        let measurement1 = ffi_utils::as_ref(measurement1);
        // TODO: Should be checking domain, not just carrier. Need to add fields to Ffi struct.
        assert_eq!(transformation0.output_domain_carrier, measurement1.input_domain_carrier);
        let input_domain_carrier = transformation0.input_domain_carrier.clone();
        let output_domain_carrier = measurement1.output_domain_carrier.clone();
        let measurement = super::make_chain_mt_alt(&measurement1.value, &transformation0.value);
        FfiMeasurementAlt::new_typed(input_domain_carrier, output_domain_carrier, measurement)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_chain_tt(transformation1: *mut FfiTransformation, transformation0: *mut FfiTransformation) -> *mut FfiTransformation {
        let transformation0 = ffi_utils::as_ref(transformation0);
        let transformation1 = ffi_utils::as_ref(transformation1);
        assert_eq!(transformation0.output_type, transformation1.input_type);
        let input_type = transformation0.input_type.clone();
        let output_type = transformation1.output_type.clone();
        let transformation = super::make_chain_tt(&transformation1.value, &transformation0.value);
        FfiTransformation::new_typed(input_type, output_type, transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_chain_tt_alt(transformation1: *mut FfiTransformationAlt, transformation0: *mut FfiTransformationAlt) -> *mut FfiTransformationAlt {
        let transformation0 = ffi_utils::as_ref(transformation0);
        let transformation1 = ffi_utils::as_ref(transformation1);
        // TODO: Should be checking domain, not just carrier. Need to add fields to Ffi struct.
        assert_eq!(transformation0.output_domain_carrier, transformation1.input_domain_carrier);
        let input_domain_carrier = transformation0.input_domain_carrier.clone();
        let output_domain_carrier = transformation1.output_domain_carrier.clone();
        let transformation = super::make_chain_tt_alt(&transformation1.value, &transformation0.value);
        FfiTransformationAlt::new_typed(input_domain_carrier, output_domain_carrier, transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_composition(measurement0: *mut FfiMeasurement, measurement1: *mut FfiMeasurement) -> *mut FfiMeasurement {
        let measurement0 = ffi_utils::as_ref(measurement0);
        let measurement1 = ffi_utils::as_ref(measurement1);
        assert_eq!(measurement0.input_type, measurement1.input_type);
        let input_type = measurement0.input_type.clone();
        let output_type = Type::new_box_pair(&measurement0.output_type, &measurement1.output_type);
        let measurement = make_composition(&measurement0.value, &measurement1.value);
        FfiMeasurement::new_typed(input_type, output_type, measurement)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_composition_alt(measurement0: *mut FfiMeasurementAlt, measurement1: *mut FfiMeasurementAlt) -> *mut FfiMeasurementAlt {
        let measurement0 = ffi_utils::as_ref(measurement0);
        let measurement1 = ffi_utils::as_ref(measurement1);
        // TODO: Should be checking domain, not just carrier. Need to add fields to Ffi struct.
        assert_eq!(measurement0.input_domain_carrier, measurement1.input_domain_carrier);
        let input_domain_carrier = measurement0.input_domain_carrier.clone();
        let output_domain_carrier = Type::new_box_pair(&measurement0.output_domain_carrier, &measurement1.output_domain_carrier);
        let measurement = make_composition_alt(&measurement0.value, &measurement1.value);
        FfiMeasurementAlt::new_typed(input_domain_carrier, output_domain_carrier, measurement)
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
    use crate::dom::{AllDomain, AllDomainAlt};

    use super::*;

    #[test]
    fn test_identity() {
        let identity = Transformation::<i32, i32>::new(AllDomain::<i32>::new(), AllDomain::<i32>::new(), |arg: &i32| arg.clone());
        let arg = 99;
        let ret = identity.invoke(&arg);
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_identity_alt() {
        let identity = TransformationAlt::new(AllDomainAlt::<i32>::new(), AllDomainAlt::<i32>::new(), |arg: &i32| arg.clone());
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
        let ret = chain.invoke(&arg);
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_chain_mt_alt() {
        let transformation = TransformationAlt::new(AllDomainAlt::<u8>::new(), AllDomainAlt::<i32>::new(), |a: &u8| (a + 1) as i32);
        let measurement = MeasurementAlt::new(AllDomainAlt::<i32>::new(), AllDomainAlt::<f64>::new(), |a: &i32| (a + 1) as f64);
        let chain = make_chain_mt_alt(&measurement, &transformation);
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
        let ret = chain.invoke(&arg);
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_chain_tt_alt() {
        let transformation0 = TransformationAlt::new(AllDomainAlt::<u8>::new(), AllDomainAlt::<i32>::new(), |a: &u8| (a + 1) as i32);
        let transformation1 = TransformationAlt::new(AllDomainAlt::<i32>::new(), AllDomainAlt::<f64>::new(), |a: &i32| (a + 1) as f64);
        let chain = make_chain_tt_alt(&transformation1, &transformation0);
        let arg = 99_u8;
        let ret = chain.function.eval(&arg);
        assert_eq!(ret, 101.0);
    }

    #[test]
    fn test_make_composition() {
        let measurement0 = Measurement::new(AllDomain::<i32>::new(), AllDomain::<f32>::new(), |arg: &i32| (arg + 1) as f32);
        let measurement1 = Measurement::new(AllDomain::<i32>::new(), AllDomain::<f64>::new(), |arg: &i32| (arg - 1) as f64);
        let chain = make_composition(&measurement0, &measurement1);
        let arg = 99;
        let ret = chain.invoke(&arg);
        assert_eq!(ret, (Box::new(100_f32), Box::new(98_f64)));
    }

    #[test]
    fn test_make_composition_alt() {
        let measurement0 = MeasurementAlt::new(AllDomainAlt::<i32>::new(), AllDomainAlt::<f32>::new(), |arg: &i32| (arg + 1) as f32);
        let measurement1 = MeasurementAlt::new(AllDomainAlt::<i32>::new(), AllDomainAlt::<f64>::new(), |arg: &i32| (arg - 1) as f64);
        let composition = make_composition_alt(&measurement0, &measurement1);
        let arg = 99;
        let ret = composition.function.eval(&arg);
        assert_eq!(ret, (Box::new(100_f32), Box::new(98_f64)));
    }

}


// PLACEHOLDERS
mod dummy {
    use super::*;
    use std::marker::PhantomData;

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
    impl<T: 'static> DomainAlt for DummyDomain<T> {
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
