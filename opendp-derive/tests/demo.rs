use opendp_derive::Mappable;
use num::Zero;

#[derive(Debug, Mappable)]
enum ExampleEnum {
    A(f64),
    B(i64)
}

impl From<f64> for ExampleEnum {
    fn from(v: f64) -> Self {
        ExampleEnum::A(v)
    }
}
impl From<i64> for ExampleEnum {
    fn from(v: i64) -> Self {
        ExampleEnum::B(v)
    }
}

// this is no longer generic after applying auxiliary parameters
// fn make_clamp_fn<T: PartialOrd + Clone>(l: T, u: T) -> impl Fn(T) -> T {
//     move |v: T| if v < l {l.clone()} else if v > u {u.clone()} else {v}
// }

fn sign<T: PartialOrd + Clone + Zero>(v: T) -> i64 {
    if v >= T::zero() {1} else {0}
}

#[test]
fn test_basic() {
    let value = ExampleEnum::A(2.);
    // let function = make_clamp_fn(0.0f64, 1.0f64);
    // let clamped_value: ExampleEnum = map_example_enum_unary!(value, function);

    let sign_value: ExampleEnum = map_example_enum_unary!(value, sign);

    println!("{:?}", sign_value);
}