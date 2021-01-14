use std::rc::Rc;

use crate::data::TraitObject;
use crate::dom::AllDomain;
use crate::ffi_utils;


// BUILDING BLOCKS
pub trait Distance {}  // PLACEHOLDER

pub trait Metric {}    // PLACEHOLDER

pub trait Measure {}   // PLACEHOLDER

pub trait PrivacyRelation {
    fn evaluate(&self, input_distance: &dyn Distance, output_distance: &dyn Distance) -> bool;
}

pub trait StabilityRelation {
    fn evaluate(&self, input_distance: &dyn Distance, output_distance: &dyn Distance) -> bool;
}

pub trait Domain: TraitObject {
    type Carrier;
    fn box_clone(&self) -> Box<dyn Domain<Carrier=Self::Carrier>>;
    fn check_compatible(&self, other: &dyn Domain<Carrier=Self::Carrier>) -> bool;
    fn check_valid(&self, val: &Self::Carrier) -> bool;
}


// MEASUREMENTS & TRANSFORMATIONS
pub struct Measurement<I, O> {
    pub input_domain: Box<dyn Domain<Carrier=I>>,
    pub input_metric: Box<dyn Metric>,
    pub output_domain: Box<dyn Domain<Carrier=O>>,
    pub output_measure: Box<dyn Measure>,
    pub privacy_relation: Box<dyn PrivacyRelation>,
    function: Rc<dyn Fn(*const I) -> Box<O>>,
}

impl<I, O> Measurement<I, O> {
    pub fn new(input_domain: impl Domain<Carrier=I> + 'static, output_domain: impl Domain<Carrier=O> + 'static, function: impl Fn(&I) -> O + 'static) -> Measurement<I, O> {
        let function = move |arg: *const I| -> Box<O> {
            let arg = ffi_utils::as_ref(arg);
            let res = function(arg);
            Box::new(res)
        };
        Measurement {
            input_domain: Box::new(input_domain),
            input_metric: Box::new(dummy::DummyMetric),
            output_domain: Box::new(output_domain),
            output_measure: Box::new(dummy::DummyMeasure),
            privacy_relation: Box::new(dummy::DummyPrivacyRelation),
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

pub struct Transformation<I, O> {
    pub input_domain: Box<dyn Domain<Carrier=I>>,
    pub input_metric: Box<dyn Metric>,
    pub output_domain: Box<dyn Domain<Carrier=O>>,
    pub output_metric: Box<dyn Metric>,
    pub stability_relation: Box<dyn StabilityRelation>,
    function: Rc<dyn Fn(*const I) -> Box<O>>,
}

impl<I, O> Transformation<I, O> {
    pub fn new(input_domain: impl Domain<Carrier=I> + 'static, output_domain: impl Domain<Carrier=O> + 'static, function: impl Fn(&I) -> O + 'static) -> Transformation<I, O> {
        let function = move |arg: *const I| -> Box<O> {
            let arg = ffi_utils::as_ref(arg);
            let res = function(arg);
            Box::new(res)
        };
        Transformation {
            input_domain: Box::new(input_domain),
            input_metric: Box::new(dummy::DummyMetric),
            output_domain: Box::new(output_domain),
            output_metric: Box::new(dummy::DummyMetric),
            stability_relation: Box::new(dummy::DummyStabilityRelation),
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
    let output_domain = measurement.output_domain.box_clone();
    let function0 = transformation.function.clone();
    let function1 = measurement.function.clone();
    let function = move |arg: *const I| -> Box<O> {
        let res0 = function0(arg);
        function1(&*res0)
    };
    let function = Box::new(function);
    Measurement {
        input_domain: input_domain,
        input_metric: Box::new(dummy::DummyMetric),
        output_domain: output_domain,
        output_measure: Box::new(dummy::DummyMeasure),
        privacy_relation: Box::new(dummy::DummyPrivacyRelation),
        function: Rc::new(function)
    }
}

pub fn make_chain_tt<I: 'static, X: 'static, O: 'static>(transformation1: &Transformation<X, O>, transformation0: &Transformation<I, X>) -> Transformation<I, O> {
    assert!(transformation0.output_domain.check_compatible(transformation1.input_domain.as_ref()));
    let input_domain = transformation0.input_domain.box_clone();
    let output_domain = transformation1.output_domain.box_clone();
    let function0 = transformation0.function.clone();
    let function1 = transformation1.function.clone();
    let function = move |arg: *const I| -> Box<O> {
        let res0 = function0(arg);
        function1(&*res0)
    };
    let function = Box::new(function);
    Transformation {
        input_domain: input_domain,
        input_metric: Box::new(dummy::DummyMetric),
        output_domain: output_domain,
        output_metric: Box::new(dummy::DummyMetric),
        stability_relation: Box::new(dummy::DummyStabilityRelation),
        function: Rc::new(function)
    }
}

pub fn make_composition<I: 'static, OA: 'static, OB: 'static>(measurement0: &Measurement<I, OA>, measurement1: &Measurement<I, OB>) -> Measurement<I, (Box<OA>, Box<OB>)> {
    assert!(measurement0.input_domain.check_compatible(measurement1.input_domain.as_ref()));
    let input_domain = measurement0.input_domain.box_clone();
    let _output_domain0 = measurement0.output_domain.box_clone();
    let _output_domain1 = measurement1.output_domain.box_clone();
    // FIXME: Figure out output_domain for composition.
    let output_domain = Box::new(AllDomain::<(Box<OA>, Box<OB>)>::new());
    let function0 = measurement0.function.clone();
    let function1 = measurement1.function.clone();
    let function = move |arg: *const I| -> Box<(Box<OA>, Box<OB>)> {
        let res0 = function0(arg);
        let res1 = function1(arg);
        Box::new((res0, res1))
    };
    Measurement {
        input_domain: input_domain,
        input_metric: Box::new(dummy::DummyMetric),
        output_domain: output_domain,
        output_measure: Box::new(dummy::DummyMeasure),
        privacy_relation: Box::new(dummy::DummyPrivacyRelation),
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
        let identity = Transformation::new(AllDomain::<i32>::new(), AllDomain::<i32>::new(), |arg: &i32| arg.clone());
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

    // pub struct DummyDistance;
    // impl Distance for DummyDistance {}
    pub struct DummyMetric;
    impl Metric for DummyMetric {}
    pub struct DummyMeasure;
    impl Measure for DummyMeasure {}
    pub struct DummyPrivacyRelation;
    impl PrivacyRelation for DummyPrivacyRelation { fn evaluate(&self, _input_distance: &dyn Distance, _output_distance: &dyn Distance) -> bool { false } }
    pub struct DummyStabilityRelation;
    impl StabilityRelation for DummyStabilityRelation { fn evaluate(&self, _input_distance: &dyn Distance, _output_distance: &dyn Distance) -> bool { false } }
}
