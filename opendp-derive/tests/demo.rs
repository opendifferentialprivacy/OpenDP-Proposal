use opendp_derive::{Apply, AutoFrom, AutoGet};
use num::{Zero, CheckedAdd};
use std::ops::Add;

#[derive(Debug)]
enum Error {
    AtomicMismatch,
    Overflow
}
#[derive(Clone, Debug, Apply, AutoFrom, AutoGet)]
enum NumericScalar {
    F64(f64),
    I64(i64),
}

#[derive(Clone, Debug, Apply, AutoFrom, AutoGet)]
enum IntScalar {
    I64(i64),
    I32(i32),
    I16(i16),
    I8(i8),
}

#[derive(Clone, Debug, Apply, AutoFrom, AutoGet)]
enum Scalar {
    #[reapply]
    Int(IntScalar),
    Bool(bool),
    String(String)
}

// this is no longer generic after applying auxiliary parameters
// fn make_clamp_fn<T: PartialOrd + Clone>(l: T, u: T) -> impl Fn(T) -> T {
//     move |v: T| if v < l {l.clone()} else if v > u {u.clone()} else {v}
// }

fn sign<T: PartialOrd + Clone + Zero>(v: T, aux: f64) -> Result<i64, Error> {
    println!("{}", aux);
    Ok(if v >= T::zero() {1} else {0})
}

fn add<T: Add<Output=T>>(l: T, r: T) -> Result<T, Error> {
    Ok(l + r)
}

fn checked_add<T: CheckedAdd<Output=T>>(l: T, r: T) -> Result<T, Error> {
    l.checked_add(&r).ok_or_else(|| Error::Overflow)
}

fn to_string<T: ToString>(x: T) -> Result<String, Error> {
    Ok(x.to_string())
}


#[test]
fn test_basic() {
    let value: NumericScalar = 2.0.into();
    println!("Wrapped value: {:?}", value);
    println!("Unwrapped value: {}", value.clone().f64().unwrap());

    let sign_value: Result<NumericScalar, Error> = apply_numeric_scalar!(sign, value; 2.);
    let sign: i64 = sign_value.unwrap().i64().unwrap();
    println!("sign: {:?}", sign);

    let sum_value: Result<NumericScalar, Error> = apply_numeric_scalar!(add, 1.0.into(), 2.0.into());
    let sum: f64 = sum_value.unwrap().f64().unwrap();
    println!("sum: {:?}", sum);

    let a: IntScalar = 1.into();
    let b: IntScalar = 2.into();
    let checked_sum: Result<IntScalar, Error> = apply_int_scalar!(checked_add, a, b);
    println!("checked sum: {:?}", checked_sum);

    let a: Scalar = IntScalar::from(1).into();
    let str_casted: String = apply_scalar!(to_string, a).unwrap();
    println!("casted: {}", str_casted);
}