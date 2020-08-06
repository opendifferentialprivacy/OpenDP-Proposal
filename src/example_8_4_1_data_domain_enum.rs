// enum is necessary to check equality
// issue: combinatorial types
// issue: cannot check valid_item for Vector, because of unknown type
//        can we just drop the trait?

trait Domain<T>: PartialEq {
    fn valid_item(self, x: T) -> bool;
}

#[derive(PartialEq)]
pub(crate) enum DataDomain {
    F64(F64),
    BoundedF64(BoundedF64),
    OptionalF64(OptionalF64),
    BoundedOptionalF64(BoundedOptionalF64),
    F32(F32),
    BoundedF32(BoundedF32),
    OptionalF32(OptionalF32),
    BoundedOptionalF32(BoundedOptionalF32),
    Vector(Vector<i64>)
}

#[derive(PartialEq)]
struct F64;

#[derive(PartialEq)]
struct BoundedF64(f64, f64);

#[derive(PartialEq)]
struct OptionalF64;

#[derive(PartialEq)]
struct BoundedOptionalF64(f64, f64);

#[derive(PartialEq)]
struct F32;

#[derive(PartialEq)]
struct BoundedF32(f32, f32);

#[derive(PartialEq)]
struct OptionalF32;

#[derive(PartialEq)]
struct BoundedOptionalF32(f32, f32);

#[derive(PartialEq)]
struct Vector<TLength: PartialEq> {
    atomic_type: DataDomain,
    is_empty: bool,
    length: Option<TLength>
}


impl Domain<f64> for F64 {
    fn valid_item(self, _x: f64) -> bool {
        true
    }
}

impl Domain<f64> for BoundedF64 {
    fn valid_item(self, x: f64) -> bool {
        self.0 < x && x < self.1
    }
}

// impl<T> Domain<TLength: PartialEq> for Vector<TLength> {
//     fn valid_item(self, x: T) -> bool {
//         self.atomic_type
//     }
// }
