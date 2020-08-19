

// adding one more trait to a sum type is difficult, I have not fully worked that out
trait NonNull {}
trait HasLowerBound {}
trait HasUpperBound {}


trait MakeBounded<T, U> {
    fn make_bounded<T, U>(input: T) -> U;
}

trait Float {}
trait BoundedFloat: HasLowerBound + HasUpperBound {
    fn make_non_null() -> dyn BoundedFloatNonNull;
}

trait BoundedFloatNonNull: NonNull + HasLowerBound + HasUpperBound {
}
trait FloatNonNull: NonNull {}


fn clamp<T: MakeBounded<T, U>, U: BoundedFloat>(data: T) -> U {
    data::make_bounded()
}

fn impute<T, U: NonNull>(data: T) -> U {

}









impl HasNullity for BoundedFloatNonNull {}
impl HasLowerBound for BoundedFloatNonNull {}
impl HasUpperBound for BoundedFloatNonNull {}

