use std::rc::Rc;

use crate::data::TraitObject;
use crate::dom::AllDomain;
use crate::ffi_utils;


// BUILDING BLOCKS
pub trait Distance {}  // PLACEHOLDER

// // Option 1: associated type
// pub trait Metric {
//     type MyDistance: Distance;
// }    // PLACEHOLDER
//
// struct L1<T> {
// }
// impl<T: Distance> Metric for L1<T> {
//     type MyDistance = T;
// }
//
// pub trait Measure {
//     type MyDistance: Distance;
// }   // PLACEHOLDER
//
// struct Hamming<T> { distance: PhantomData<T> }
// impl<T> Metric for Hamming<T> { type MyDistance = T; }
//
// struct Symmetric<T> {distance: PhantomData<T> }
// impl<T> Metric for Symmetric<T> { type MyDistance = T; }
//
// // Option 2: enum type
// // is it really any better than option 1?
// enum Metric<T> {
//     Hamming(T), Symmetric(T),
// }

// // Option 3: completely erase type
// // Will be difficult to downcast from
// // could potentially use the generic type of the data to only downcast once (for eg. sum relation)
// pub trait Metric {}


// Option 4: Metrics are instances of single struct, distinguished by identity
// Have to do comparison by pointer checks
// struct Metric<T: Distance> {
//     value: PhantomData<T>
// }
//
// static HAMMING: Metric<i32> = Metric {value: PhantomData()};
// static SYMMETRIC: Metric<i32> = Metric {value: PhantomData<i32>};
// static L1_INT32: Metric<i32> = Metric {value: PhantomData<i32>};
// static L1_FLOAT64: Metric<f32> = Metric {value: PhantomData<i32>};

pub trait Metric {
    type Distance;
    fn box_clone(&self) -> Box<dyn Metric<Distance=Self::Distance>>;
}

pub trait Measure {
    type Distance;
    fn box_clone(&self) -> Box<dyn Measure<Distance=Self::Distance>>;
}

pub trait PrivacyRelation {
    type InputDistance;
    type OutputDistance;
    fn evaluate(&self, input_distance: &Self::InputDistance, output_distance: &Self::OutputDistance) -> bool;
}

pub trait StabilityRelation {
    type InputDistance;
    type OutputDistance;
    fn evaluate(&self, input_distance: &Self::InputDistance, output_distance: &Self::OutputDistance) -> bool;
}

pub trait Domain: TraitObject {
    type Carrier;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>>;
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool;
    fn check_valid(&self, val: &Self::Carrier) -> bool;
}


// MEASUREMENTS & TRANSFORMATIONS
pub struct Measurement<I, O, ID=(), OD=()> {
    pub input_domain: Box<dyn Domain<Carrier=I>>,
    pub input_metric: Box<dyn Metric<Distance=ID>>,
    pub output_domain: Box<dyn Domain<Carrier=O>>,
    pub output_measure: Box<dyn Measure<Distance=OD>>,
    pub privacy_relation: Box<dyn PrivacyRelation<InputDistance=ID, OutputDistance=OD>>,
    function: Rc<dyn Fn(*const I) -> Box<O>>,
}

impl<I, O, ID: 'static, OD: 'static> Measurement<I, O, ID, OD> {
    pub fn new(input_domain: impl Domain<Carrier=I> + 'static, output_domain: impl Domain<Carrier=O> + 'static, function: impl Fn(&I) -> O + 'static) -> Self {
        let input_metric = dummy::DummyMetric::<ID>::new();
        let output_measure = dummy::DummyMeasure::<OD>::new();
        let privacy_relation = dummy::DummyPrivacyRelation::new();
        Self::new_all(input_domain, input_metric, output_domain, output_measure, privacy_relation, function)
    }

    pub fn new_all(
        input_domain: impl Domain<Carrier=I> + 'static,
        input_metric: impl Metric<Distance=ID> + 'static,
        output_domain: impl Domain<Carrier=O> + 'static,
        output_measure: impl Measure<Distance=OD> + 'static,
        privacy_relation: impl PrivacyRelation<InputDistance=ID, OutputDistance=OD> + 'static,
        function: impl Fn(&I) -> O + 'static
    ) -> Measurement<I, O, ID, OD> {
        let function = move |arg: *const I| -> Box<O> {
            let arg = ffi_utils::as_ref(arg);
            let res = function(arg);
            Box::new(res)
        };
        Measurement {
            input_domain: Box::new(input_domain),
            input_metric: Box::new(input_metric),
            output_domain: Box::new(output_domain),
            output_measure: Box::new(output_measure),
            privacy_relation: Box::new(privacy_relation),
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

pub struct Transformation<I, O, ID=(), OD=()> {
    pub input_domain: Box<dyn Domain<Carrier=I>>,
    pub input_metric: Box<dyn Metric<Distance=ID>>,
    pub output_domain: Box<dyn Domain<Carrier=O>>,
    pub output_metric: Box<dyn Metric<Distance=OD>>,
    pub stability_relation: Box<dyn StabilityRelation<InputDistance=ID, OutputDistance=OD>>,
    function: Rc<dyn Fn(*const I) -> Box<O>>,
}

impl<I, O, ID: 'static, OD: 'static> Transformation<I, O, ID, OD> {
    pub fn new(input_domain: impl Domain<Carrier=I> + 'static, output_domain: impl Domain<Carrier=O> + 'static, function: impl Fn(&I) -> O + 'static) -> Transformation<I, O, ID, OD> {
        let input_metric = dummy::DummyMetric::<ID>::new();
        let output_metric = dummy::DummyMetric::<OD>::new();
        let stability_relation = dummy::DummyStabilityRelation::<ID, OD>::new();
        Self::new_all(input_domain, input_metric, output_domain, output_metric, stability_relation, function)
    }

    pub fn new_all(
        input_domain: impl Domain<Carrier=I> + 'static,
        input_metric: impl Metric<Distance=ID> + 'static,
        output_domain: impl Domain<Carrier=O> + 'static,
        output_metric: impl Metric<Distance=OD> + 'static,
        stability_relation: impl StabilityRelation<InputDistance=ID, OutputDistance=OD> + 'static,
        function: impl Fn(&I) -> O + 'static
    ) -> Self {
        let function = move |arg: *const I| -> Box<O> {
            let arg = ffi_utils::as_ref(arg);
            let res = function(arg);
            Box::new(res)
        };
        Transformation {
            input_domain: Box::new(input_domain),
            input_metric: Box::new(input_metric),
            output_domain: Box::new(output_domain),
            output_metric: Box::new(output_metric),
            stability_relation: Box::new(stability_relation),
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


// CHAINING & COMPOSITION
pub fn make_chain_mt<I: 'static, X: 'static, O: 'static>(measurement: &Measurement<X, O>, transformation: &Transformation<I, X>) -> Measurement<I, O> {
    assert!(transformation.output_domain.check_compatible(measurement.input_domain.as_ref()));
    let input_domain = transformation.input_domain.box_clone();
    let input_metric = transformation.input_metric.box_clone();
    let output_domain = measurement.output_domain.box_clone();
    let output_measure = measurement.output_measure.box_clone();
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
        privacy_relation: Box::new(dummy::DummyPrivacyRelation::new()),
        function: Rc::new(function)
    }
}

pub fn make_chain_tt<I: 'static, X: 'static, O: 'static>(transformation1: &Transformation<X, O>, transformation0: &Transformation<I, X>) -> Transformation<I, O> {
    assert!(transformation0.output_domain.check_compatible(transformation1.input_domain.as_ref()));
    let input_domain = transformation0.input_domain.box_clone();
    let input_metric = transformation0.input_metric.box_clone();
    let output_domain = transformation1.output_domain.box_clone();
    let output_metric = transformation1.output_metric.box_clone();
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
        stability_relation: Box::new(dummy::DummyStabilityRelation::new()),
        function: Rc::new(function)
    }
}

pub fn make_composition<I: 'static, OA: 'static, OB: 'static>(measurement0: &Measurement<I, OA>, measurement1: &Measurement<I, OB>) -> Measurement<I, (Box<OA>, Box<OB>)> {
    assert!(measurement0.input_domain.check_compatible(measurement1.input_domain.as_ref()));
    let input_domain = measurement0.input_domain.box_clone();
    // TODO: Figure out input_metric for composition.
    let input_metric = measurement0.input_metric.box_clone();
    let _output_domain0 = measurement0.output_domain.box_clone();
    let _output_domain1 = measurement1.output_domain.box_clone();
    // TODO: Figure out output_domain for composition.
    let output_domain = Box::new(AllDomain::<(Box<OA>, Box<OB>)>::new());
    // TODO: Figure out output_measure for composition.
    let output_measure = measurement0.output_measure.box_clone();
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
        privacy_relation: Box::new(dummy::DummyPrivacyRelation::new()),
        function: Rc::new(function)
    }
}


// FFI BINDINGS
pub(crate) mod ffi {
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
    pub extern "C" fn opendp_core__measurement_free(this: *mut FfiMeasurement) {
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
    pub extern "C" fn opendp_core__transformation_free(this: *mut FfiTransformation) {
        ffi_utils::into_owned(this);
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_chain_mt(measurement1: *mut FfiMeasurement, transformation0: *mut FfiTransformation) -> *mut FfiMeasurement {
        let transformation0 = ffi_utils::into_owned(transformation0);
        let measurement1 = ffi_utils::into_owned(measurement1);
        assert_eq!(transformation0.output_type, measurement1.input_type);
        let input_type = transformation0.input_type.clone();
        let output_type = measurement1.output_type.clone();
        let measurement = super::make_chain_mt(&measurement1.value, &transformation0.value);
        FfiMeasurement::new_typed(input_type, output_type, measurement)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_chain_tt(transformation1: *mut FfiTransformation, transformation0: *mut FfiTransformation) -> *mut FfiTransformation {
        let transformation0 = ffi_utils::into_owned(transformation0);
        let transformation1 = ffi_utils::into_owned(transformation1);
        assert_eq!(transformation0.output_type, transformation1.input_type);
        let input_type = transformation0.input_type.clone();
        let output_type = transformation1.output_type.clone();
        let transformation = super::make_chain_tt(&transformation1.value, &transformation0.value);
        FfiTransformation::new_typed(input_type, output_type, transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_core__make_composition(measurement0: *mut FfiMeasurement, measurement1: *mut FfiMeasurement) -> *mut FfiMeasurement {
        let measurement0 = ffi_utils::into_owned(measurement0);
        let measurement1 = ffi_utils::into_owned(measurement1);
        assert_eq!(measurement0.input_type, measurement1.input_type);
        let input_type = measurement0.input_type.clone();
        let output_type = Type::new_box_pair(&measurement0.output_type, &measurement1.output_type);
        let measurement = make_composition(&measurement0.value, &measurement1.value);
        FfiMeasurement::new_typed(input_type, output_type, measurement)
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
        let identity = Transformation::<i32, i32, (), ()>::new(AllDomain::<i32>::new(), AllDomain::<i32>::new(), |arg: &i32| arg.clone());
        let arg = 99;
        let ret = identity.invoke(&arg);
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
    fn test_make_chain_tt() {
        let transformation0 = Transformation::new(AllDomain::<u8>::new(), AllDomain::<i32>::new(), |a: &u8| (a + 1) as i32);
        let transformation1 = Transformation::new(AllDomain::<i32>::new(), AllDomain::<f64>::new(), |a: &i32| (a + 1) as f64);
        let chain = make_chain_tt(&transformation1, &transformation0);
        let arg = 99_u8;
        let ret = chain.invoke(&arg);
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

}


// PLACEHOLDERS
mod dummy {
    use super::*;
    use std::marker::PhantomData;

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
        fn box_clone(&self) -> Box<dyn Metric<Distance=Self::Distance>> { Box::new(self.clone()) }
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
        fn box_clone(&self) -> Box<dyn Measure<Distance=Self::Distance>> { Box::new(self.clone()) }
    }

    pub struct DummyPrivacyRelation<I, O> {
        _marker_i: PhantomData<I>,
        _marker_o: PhantomData<O>,
    }
    impl<I, O> DummyPrivacyRelation<I, O> {
        pub fn new() -> Self {
            DummyPrivacyRelation { _marker_i: PhantomData, _marker_o: PhantomData }
        }
    }
    impl<I, O> Clone for DummyPrivacyRelation<I, O> {
        fn clone(&self) -> Self { Self::new() }
    }
    impl<I, O> PrivacyRelation for DummyPrivacyRelation<I, O> {
        type InputDistance = I;
        type OutputDistance = O;
        fn evaluate(&self, _input_distance: &Self::InputDistance, _output_distance: &Self::OutputDistance) -> bool { false }
    }

    pub struct DummyStabilityRelation<I, O> {
        _marker_i: PhantomData<I>,
        _marker_o: PhantomData<O>,
    }
    impl<I, O> DummyStabilityRelation<I, O> {
        pub fn new() -> Self {
            DummyStabilityRelation { _marker_i: PhantomData, _marker_o: PhantomData }
        }
    }
    impl<I, O> Clone for DummyStabilityRelation<I, O> {
        fn clone(&self) -> Self { Self::new() }
    }
    impl<I, O> StabilityRelation for DummyStabilityRelation<I, O> {
        type InputDistance = I;
        type OutputDistance = O;
        fn evaluate(&self, _input_distance: &Self::InputDistance, _output_distance: &Self::OutputDistance) -> bool { false }
    }
}
