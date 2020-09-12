use opendp_derive::{Apply, AutoFrom};
use num::Zero;

#[derive(Debug, Apply, AutoFrom)]
enum NumericScalar {
    F64(f64),
    I64(i64),
}

// this is no longer generic after applying auxiliary parameters
// fn make_clamp_fn<T: PartialOrd + Clone>(l: T, u: T) -> impl Fn(T) -> T {
//     move |v: T| if v < l {l.clone()} else if v > u {u.clone()} else {v}
// }

fn sign<T: PartialOrd + Clone + Zero>(v: T, aux: f64) -> i64 {
    println!("{}", aux);
    if v >= T::zero() {1} else {0}
}

#[test]
fn test_basic() {
    let value = NumericScalar::from(2.);
    // let function = make_clamp_fn(0.0f64, 1.0f64);
    // let clamped_value: ExampleEnum = map_example_enum_unary!(value, function);

    let sign_value: NumericScalar = apply_numeric_scalar!(sign, value; 2.);

    // println!("{:?}", sign_value);
}