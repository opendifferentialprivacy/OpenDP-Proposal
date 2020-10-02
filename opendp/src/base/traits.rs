use noisy_float::types::{R32, R64}; // , r64, r32
use num::Float;

macro_rules! impl_trait_through_method {
    ($name:ident, $new_method:ident, $old_method:ident, $( $typ:ty),*) => {
        $(
            impl $name for $typ {
                fn $new_method(self) -> Self {
                    self.$old_method()
                }
            }
        )*
    }
}
/// Unified trait for computing the absolute value
/// .abs() is inconsistent across types.
/// Implementations are scattered across methods, Abs, Float, or missing
pub trait OmniAbs {
    fn omni_abs(self) -> Self;
}
impl_trait_through_method!(OmniAbs, omni_abs, abs, f32, f64, R32, R64, i8, i16, i32, i64, i128);
macro_rules! impl_unsigned_abs {
    ($($typ:ty),*) => ($(impl OmniAbs for $typ {fn omni_abs(self) -> Self {self}})*)
}
impl_unsigned_abs!(u8, u16, u32, u64, u128);

// pub trait Impute {
//     type Output;
//     fn impute(self, l: Self::Output, u: Self::Output) -> Self::Output;
// }
// macro_rules! impl_impute_float {
//     ($ty_in:ty, $ty_out:ty, $cast:ident) => {
//         impl Impute for $ty_in {
//             type Output = $ty_out;
//             fn impute(self, l: Self::Output, u: Self::Output) -> Self::Output {
//                 if self.is_nan() { (u + l) / $cast(2.) } else {$cast(self)}
//             }
//         }
//     }
// }
// impl_impute_float!(f64, R64, r64);
// impl_impute_float!(f32, R32, r32);
//
// macro_rules! impl_impute_int {
//     ($($ty_out:ty),*) => {
//         $(
//         impl Impute for Option<$ty_out> {
//             type Output = $ty_out;
//             fn impute(self, l: $ty_out, u: $ty_out) -> Self::Output {
//                 self.unwrap_or((u + l) / 2)
//             }
//         }
//         )*
//     }
// }
// impl_impute_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
