
#[derive(PartialEq, Clone)]
pub(crate) enum DataDomain {
    Scalar(Scalar),
    Vector(Vector<i64>)
}


#[derive(PartialEq, Clone)]
pub(crate) struct Scalar(pub(crate) AtomicDomain);

#[derive(PartialEq, Clone)]
pub(crate) struct Vector<TLength: PartialEq + Clone> {
    pub(crate) atomic_type: Box<DataDomain>,
    pub(crate) is_empty: bool,
    pub(crate) length: Option<TLength>
}

#[derive(PartialEq, Clone)]
pub(crate) enum AtomicDomain {
    Float(FloatDomain),
    Int(IntDomain),
    OptionalInt(OptionalIntDomain),
    Str(StrDomain),
    Bool(BoolDomain)
}

#[derive(PartialEq, Clone)]
pub(crate) enum FloatDomain {
    F64(F64Domain),
    F32(F32Domain),
}

#[derive(PartialEq, Clone)]
pub(crate) enum IntDomain {
    I64(I64Domain),
    I32(I32Domain),
    I16(I16Domain),
    I8(I8Domain),
}

#[derive(PartialEq, Clone)]
pub(crate) enum OptionalIntDomain {
    OptionalI64(OptionalI64Domain),
    OptionalI32(OptionalI32Domain),
    OptionalI16(OptionalI16Domain),
    OptionalI8(OptionalI8Domain),
}



macro_rules! derive_float {
    ($name:ident, $ty:ty) => {

        #[derive(PartialEq, Clone)]
        pub(crate) struct $name {
            pub(crate) non_null: bool,
            pub(crate) lower: Option<$ty>,
            pub(crate) upper: Option<$ty>
        }

        impl $name {
            fn is_valid(&self, x: $ty) -> bool {
                if self.non_null && !x.is_finite() {
                    return false
                }

                if self.lower.map(|l| l.is_finite() && l > x).unwrap_or(false) {
                    return false
                }

                if self.upper.map(|u| u.is_finite() && u < x).unwrap_or(false) {
                    return false
                }

                true
            }

            fn default_value(&self) -> $ty {
                match (self.non_null, self.lower, self.upper) {
                    (false, _, _) => <$ty>::NAN,
                    (_, Some(l), Some(u)) => (l + u) / 2.,
                    (_, Some(l), None) => l,
                    (_, None, Some(u)) => u,
                    (_, None, None) => <$ty>::default()
                }
            }
        }

    }
}


macro_rules! derive_optional_integer {
    ($name:ident, $ty:ty) => {
        #[derive(PartialEq, Clone)]
        pub(crate) struct $name {
            pub(crate) non_null: bool,
            pub(crate) lower: Option<$ty>,
            pub(crate) upper: Option<$ty>,
            pub(crate) categories: Option<Vec<$ty>>
        }

        impl $name {
            fn is_valid(&self, x: Option<$ty>) -> bool {
                if x.is_none() {
                    return !self.non_null
                }

                let x = x.unwrap();

                if self.lower.map(|l| l > x).unwrap_or(false) {
                    return false
                }

                if self.upper.map(|u| u < x).unwrap_or(false) {
                    return false
                }

                if self.categories.as_ref().map(|cats| !cats.contains(&x)).unwrap_or(false) {
                    return false
                }
                true
            }

            fn default_value(&self) -> Option<$ty> {
                if let Some(categories) = &self.categories {
                    return Some(*categories.first().unwrap())
                }
                match (self.non_null, self.lower, self.upper) {
                    (false, _, _) => None,
                    (_, Some(l), Some(u)) => Some((l + u) / 2),
                    (_, Some(l), None) => Some(l),
                    (_, None, Some(u)) => Some(u),
                    (_, None, None) => Some(<$ty>::default())
                }
            }
        }
    }
}


macro_rules! derive_integer {
    ($name:ident, $ty:ty) => {
        #[derive(PartialEq, Clone)]
        pub(crate) struct $name {
            pub(crate) lower: Option<$ty>,
            pub(crate) upper: Option<$ty>,
            pub(crate) categories: Option<Vec<$ty>>
        }

        impl $name {
            fn is_valid(&self, x: $ty) -> bool {
                if self.lower.map(|l| l > x).unwrap_or(false) {
                    return false
                }
                if self.upper.map(|u| u < x).unwrap_or(false) {
                    return false
                }
                true
            }

            fn default_value(&self) -> $ty {
                if let Some(categories) = &self.categories {
                    return categories.first().unwrap().clone()
                }
                match (self.lower, self.upper) {
                    (Some(l), Some(u)) => (l + u) / 2,
                    (Some(l), None) => l,
                    (None, Some(u)) => u,
                    (None, None) => <$ty>::default()
                }
            }
        }
    }
}

derive_float!(F64Domain, f64);
derive_float!(F32Domain, f32);

derive_optional_integer!(OptionalI64Domain, i64);
derive_optional_integer!(OptionalI32Domain, i32);
derive_optional_integer!(OptionalI16Domain, i16);
derive_optional_integer!(OptionalI8Domain, i8);

derive_integer!(I64Domain, i64);
derive_integer!(I32Domain, i32);
derive_integer!(I16Domain, i16);
derive_integer!(I8Domain, i8);


#[derive(PartialEq, Clone)]
pub(crate) struct StrDomain {
    max_length: Option<usize>,
    categories: Option<Vec<String>>
}

impl StrDomain {
    fn is_valid(&self, x: String) -> bool {
        if let Some(categories) = &self.categories {
            if !categories.contains(&x) {
                return false
            }
        }
        if let Some(max_length) = self.max_length {
            x.len() < max_length
        } else {true}
    }

    fn default_value(&self) -> String {
        if let Some(categories) = &self.categories {
            return categories.first().unwrap().clone()
        }
        String::default()
    }
}

#[derive(PartialEq, Clone)]
pub(crate) struct BoolDomain {}
impl BoolDomain {
    fn is_valid(&self, _x: bool) -> bool {
        true
    }
    fn default_value(&self) -> bool {
        false
    }
}


