use noisy_float::types::r64;
use opendp;
use opendp::{Error, Measurement};
use opendp::base::Data;
use opendp::base::domain::{Domain, Interval, Nature, ScalarDomain, VectorDomain};
use opendp::base::metric::{DataDistance, PrivacyDistance};
use opendp::base::value::{Value, Scalar};
use opendp::constructors::aggregate::{make_sum, sensitivity_symmetric};
use opendp::constructors::chain::make_mt_chain;
use opendp::constructors::mechanisms::make_base_gaussian;

use opendp_derive::apply_numeric;
use std::cmp::Ordering;
use std::mem::{discriminant, Discriminant};
use std::any::Any;

// Ethan:
// 1. If you had to sum up the current functionality, how would you describe it?
// 2. What remains to be done (i.e. what is the gap in functionality between OpenDP and ____noise),
//    and what is the highest priority for next steps?
// 3. We would benefit from thorough documentation and commenting - as of now it can be very
//    difficult to understand the different pieces, how they fit together, and how to use them. The
//    tests in this file are very helpful in that regard.

// Mike:
// 1. Tooling to build components for data pipelines.
// 2. Components to build data pipelines with.
//        A. Components
//            - domain propagation
//            - relation
//            - function
//        B. FFI support
//        C. Python library
// 3. I agree, it is necessary for us to add that as the structures become more stable. I think it's time?

fn test_measurement(meas: Measurement, data: Data, in_dist: DataDistance) -> Result<(), Error> {

    let epsilon = Scalar::from(r64(1.));
    let delta = Scalar::from(r64(0.0001));
    let out_dist = PrivacyDistance::Approximate(epsilon, delta);
    let is_ok = meas.privacy_relation(&in_dist, &out_dist)?;

    println!("is_ok: {}", is_ok);

    let release = meas.function(data, &in_dist, &out_dist)?;

    if let Data::Value(Value::Scalar(v)) = release {
        println!("release: {:?}", v);
    };
    Ok(())
}

fn example_base_gauss() -> Result<(), Error> {

    let lower = Scalar::from(r64(0.0));
    let upper = Scalar::from(r64(1000.0));

    let original_domain = Domain::Scalar(ScalarDomain {
        may_have_nullity: false,
        nature: Nature::Numeric(Interval::new(Some(lower), Some(upper))?)
    });

    let base_gauss = make_base_gaussian(original_domain.clone(), r64(20.0).into())?;
    let data = Data::Value(Value::Scalar(r64(2.).into()));
    let in_dist = DataDistance::L2Sensitivity(Scalar::from(r64(1.)));

    test_measurement(base_gauss, data, in_dist)?;

    Ok(())
}

fn example_sum() -> Result<(), Error> {
    let lower: Scalar = r64(0.0).into();
    let upper: Scalar = r64(1.0).into();
    let original_domain = Domain::Vector(VectorDomain {
        length: Some(1000),
        is_nonempty: false,
        atomic_type: Box::new(Domain::Scalar(ScalarDomain {
            may_have_nullity: false,
            nature: Nature::Numeric(Interval::new(Some(lower.clone()), Some(upper.clone()))?)
        })),
    });

    let sum = make_sum(original_domain.clone())?;

    let base_gauss = make_base_gaussian(sum.output_domain(), r64(10.0).into())?;

    let measurement = make_mt_chain(
        base_gauss, sum,
        // NOTE: neither in distance or out distance are used
        Some(Box::new(move |_in_dist: &DataDistance, _out_dist: &PrivacyDistance| {
            let scalar: Scalar = apply_numeric!(sensitivity_symmetric, lower.clone(): Scalar, upper.clone(): Scalar)?;
            Ok(DataDistance::L2Sensitivity(scalar))
        })))?;

    let raw_data = (0..1000).map(|_| r64(1.)).collect::<Vec<_>>();
    let data = Data::Value(Value::Vector(raw_data.into()));
    let in_dist = DataDistance::Hamming(1);

    test_measurement(measurement, data, in_dist)?;

    Ok(())
}

enum Scalar2 {
    F64(f64),
    I64(i64)
}

enum TransVariants {
    F64_F64 = 1,
    I64_I64 = 2
}

struct Transformation<T, U> {
    function: Box<dyn Fn(T) -> U>
}

fn make_clamp<T: PartialOrd>() -> Transformation<T, T> {
    Transformation {
        function: Box::new(move |v: T| v)
    }
}

trait FFIObject {
    fn as_any(&self) -> &dyn Any;
}
impl FFIObject for Transformation<f64, f64> {
    fn as_any(&self) -> &dyn Any { self }
}
impl FFIObject for Transformation<i64, i64> {
    fn as_any(&self) -> &dyn Any { self }
}

fn main() {
    // example_base_gauss().unwrap();
    // example_sum().unwrap();

    let scalar = Scalar2::F64(2.0);
    let (discrim, boxed) = match scalar {
        Scalar2::F64(_v) => (TransVariants::F64_F64, Box::new(make_clamp::<f64>()) as Box<dyn FFIObject>),
        Scalar2::I64(_v) => (TransVariants::I64_I64, Box::new(make_clamp::<i64>()) as Box<dyn FFIObject>),
    };

    match discrim {
        TransVariants::F64_F64 => {
            boxed.as_any().downcast_ref::<Transformation<f64, f64>>();
        }
        TransVariants::I64_I64 => {
            boxed.as_any().downcast_ref::<Transformation<i64, i64>>();
        }
    }
}
