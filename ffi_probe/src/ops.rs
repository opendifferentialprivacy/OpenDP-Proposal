use std::collections::HashMap;
use std::fmt::Debug;
use std::iter;
use std::iter::Sum;
use std::ops::Bound;
use std::str::FromStr;

use rand::Rng;

use crate::core::{DomainPtr, MeasurementPtr, TransformationPtr};
use crate::data::{Data, Element, Form};
use crate::dom::{AllDomainPtr, DataDomainPtr, HeterogeneousMapDomainPtr, IntervalDomainPtr, VectorDomainPtr};


pub fn make_identity_ptr<T: 'static + Form + Clone>() -> TransformationPtr<T, T> {
    let input_domain = AllDomainPtr::<T>::new();
    let output_domain = AllDomainPtr::<T>::new();
    let function = |arg: &T| -> T {
        arg.clone()
    };
    TransformationPtr::new(input_domain, output_domain, function)
}

fn vec_string_to_str(src: &Vec<String>) -> Vec<&str> {
    src.into_iter().map(|e| e.as_str()).collect()
}

fn vec_str_to_string(src: Vec<&str>) -> Vec<String> {
    src.into_iter().map(|e| e.to_owned()).collect()
}

fn split_lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}

pub fn make_split_lines_ptr() -> TransformationPtr<String, Vec<String>> {
    let input_domain = AllDomainPtr::<String>::new();
    let output_domain = VectorDomainPtr::<String>::new_all();
    let function = |arg: &String| -> Vec<String> {
        let ret = split_lines(arg);
        vec_str_to_string(ret)
    };
    TransformationPtr::new(input_domain, output_domain, function)
}

fn parse_series<T>(col: &Vec<&str>, default_on_error: bool) -> Vec<T> where
    T: FromStr + Default,
    T::Err: Debug {
    if default_on_error {
        col.into_iter().map(|e| e.parse().unwrap_or_else(|_| T::default())).collect()
    } else {
        col.into_iter().map(|e| e.parse().unwrap()).collect()
    }
}

pub fn make_parse_series_ptr<T>(impute: bool) -> TransformationPtr<Vec<String>, Vec<T>> where
    T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
    let input_domain = VectorDomainPtr::<String>::new_all();
    let output_domain = VectorDomainPtr::<T>::new_all();
    let function = move |arg: &Vec<String>| -> Vec<T> {
        let arg = vec_string_to_str(arg);
        parse_series(&arg, impute)
    };
    TransformationPtr::new(input_domain, output_domain, function)
}

fn split_records<'a>(separator: &str, lines: &Vec<&'a str>) -> Vec<Vec<&'a str>> {
    fn split<'a>(line: &'a str, separator: &str) -> Vec<&'a str> {
        line.split(separator).into_iter().map(|e| e.trim()).collect()
    }
    lines.into_iter().map(|e| split(e, separator)).collect()
}

pub fn make_split_records_ptr(separator: Option<&str>) -> TransformationPtr<Vec<String>, Vec<Vec<String>>> {
    let separator = separator.unwrap_or(",").to_owned();
    let input_domain = VectorDomainPtr::<String>::new_all();
    let output_domain = VectorDomainPtr::<Vec<String>>::new(VectorDomainPtr::<String>::new_all());
    let function = move |arg: &Vec<String>| -> Vec<Vec<String>> {
        let arg = vec_string_to_str(arg);
        let ret = split_records(&separator, &arg);
        ret.into_iter().map(vec_str_to_string).collect()
    };
    TransformationPtr::new(input_domain, output_domain, function)
}

fn conform_records<'a>(len: usize, records: &Vec<Vec<&'a str>>) -> Vec<Vec<&'a str>> {
    fn conform<'a>(record: &Vec<&'a str>, len: usize) -> Vec<&'a str> {
        if record.len() > len {
            record[0..len].to_vec().clone()
        } else if record.len() < len {
            record.clone().into_iter().chain(iter::repeat("").take(len - record.len())).collect()
        } else {
            record.clone()
        }
    }
    records.into_iter().map(|e| conform(e, len)).collect()
}

pub type DataFrame = HashMap<String, Data>;

fn create_dataframe(col_count: usize, records: &Vec<Vec<&str>>) -> DataFrame {
    let records = conform_records(col_count, &records);
    let mut cols = vec![Vec::new(); col_count];
    for record in records.into_iter() {
        for i in 0..col_count {
            cols[i].push(record[i])
        }
    }
    cols.into_iter().enumerate().map(|(k, v)| (k.to_string(), Data::new(vec_str_to_string(v)))).collect()
}

pub fn create_raw_dataframe_domain_ptr(col_count: usize) -> impl DomainPtr<Carrier=DataFrame> {
    let element_domain = DataDomainPtr::<Vec<String>>::new(VectorDomainPtr::<String>::new_all());
    let element_domain: Box<dyn DomainPtr<Carrier=Data>> = Box::new(element_domain);
    let element_domains: HashMap<_, _> = (0..col_count).map(|e| (e.to_string(), element_domain.box_clone())).collect();
    HeterogeneousMapDomainPtr::new(element_domains)
}

pub fn make_create_dataframe_ptr(col_count: usize) -> TransformationPtr<Vec<Vec<String>>, DataFrame> {
    let input_domain = VectorDomainPtr::<Vec<String>>::new(VectorDomainPtr::<String>::new_all());
    let output_domain = create_raw_dataframe_domain_ptr(col_count);
    let function = move |arg: &Vec<Vec<String>>| -> DataFrame {
        let arg = arg.into_iter().map(|e| vec_string_to_str(e)).collect();
        create_dataframe(col_count, &arg)
    };
    TransformationPtr::new(input_domain, output_domain, function)
}

fn split_dataframe<'a>(separator: &str, col_count: usize, s: &str) -> DataFrame {
    let lines = split_lines(s);
    let records = split_records(separator, &lines);
    let records = conform_records(col_count, &records);
    create_dataframe(col_count, &records)
}

pub fn make_split_dataframe_ptr(separator: Option<&str>, col_count: usize) -> TransformationPtr<String, DataFrame> {
    let separator = separator.unwrap_or(",").to_owned();
    let input_domain = AllDomainPtr::<String>::new();
    let output_domain = create_raw_dataframe_domain_ptr(col_count);
    let function = move |arg: &String| -> DataFrame {
        split_dataframe(&separator, col_count, &arg)
    };
    TransformationPtr::new(input_domain, output_domain, function)
}

fn replace_col(key: &str, df: &DataFrame, col: &Data) -> DataFrame {
    let mut df = df.clone();
    *df.get_mut(key).unwrap() = col.clone();
    df
}

fn parse_column<T>(key: &str, impute: bool, df: &DataFrame) -> DataFrame where
    T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
    let col = df.get(key).unwrap();
    let col = col.as_form();
    let col = vec_string_to_str(col);
    let col = parse_series::<T>(&col, impute);
    replace_col(key, &df, &col.into())
}

pub fn make_parse_column_ptr<T>(input_domain: &dyn DomainPtr<Carrier=DataFrame>, key: &str, impute: bool) -> TransformationPtr<DataFrame, DataFrame> where
    T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
    let key = key.to_owned();
    let input_domain = input_domain.as_any().downcast_ref::<HeterogeneousMapDomainPtr>().expect("Bogus input_domain in make_parse_column()");
    // TODO: Assert rest of input_domain is valid.
    let input_domain = input_domain.clone();
    let make_element_domain = |k: &String, v: &Box<dyn DomainPtr<Carrier=Data>>| -> Box<dyn DomainPtr<Carrier=Data>> {
        if k == &key {
            Box::new(DataDomainPtr::<Vec<T>>::new(VectorDomainPtr::<T>::new_all()))
        } else {
            v.box_clone()
        }
    };
    let output_element_domains = input_domain.element_domains.iter().map(|(k, v)| (k.clone(), make_element_domain(k, v))).collect();
    let output_domain = HeterogeneousMapDomainPtr::new(output_element_domains);
    let function = move |arg: &DataFrame| -> DataFrame {
        parse_column::<T>(&key, impute, arg)
    };
    TransformationPtr::new(input_domain, output_domain, function)
}

pub fn make_select_column_ptr<T>(input_domain: &dyn DomainPtr<Carrier=DataFrame>, key: &str) -> TransformationPtr<DataFrame, Vec<T>> where
    T: 'static + Element + Clone + PartialEq {
    let key = key.to_owned();
    let input_domain = input_domain.as_any().downcast_ref::<HeterogeneousMapDomainPtr>().expect("Bogus input_domain in make_select_column()");
    let column_domain = input_domain.element_domains.get(&key).expect("Bogus input_domain in make_select_column()");
    let column_domain = column_domain.as_any().downcast_ref::<DataDomainPtr<Vec<T>>>().expect("Bogus input_domain in make_select_column()");
    // It's a drag that we need a type argument to get column_domain out. Might want to change Transformation::new() to take Box<dyn Domain> instead of impl Domain.
    let column_domain = column_domain.form_domain.as_any().downcast_ref::<VectorDomainPtr<T>>().expect("Bogus input_domain in make_select_column()");
    let input_domain = input_domain.clone();
    let output_domain = column_domain.clone();
    let function = move |arg: &DataFrame| -> Vec<T> {
        let ret = arg.get(&key).expect("Missing dataframe column");
        let ret: &Vec<T> = ret.as_form();
        ret.clone()
    };
    TransformationPtr::new(input_domain, output_domain, function)
}

fn clamp<T: Copy + PartialOrd>(lower: T, upper: T, x: &Vec<T>) -> Vec<T> {
    fn clamp1<T: Copy + PartialOrd>(lower: T, upper: T, x: T) -> T {
        if x < lower { lower } else if x > upper { upper } else { x }
    }
    x.into_iter().map(|e| clamp1(lower, upper, *e)).collect()

}

pub fn make_clamp_ptr<T>(input_domain: &dyn DomainPtr<Carrier=Vec<T>>, lower: T, upper: T) -> TransformationPtr<Vec<T>, Vec<T>> where
    T: 'static + Element + Copy + PartialEq + PartialOrd {
    let input_domain = input_domain.as_any().downcast_ref::<VectorDomainPtr<T>>().expect("Bogus input_domain in make_clamp()");
    let input_domain = input_domain.clone();
    let output_domain = VectorDomainPtr::<T>::new(IntervalDomainPtr::new(Bound::Included(lower), Bound::Included(upper)));
    let function = move |arg: &Vec<T>| -> Vec<T> {
        clamp(lower, upper, arg)
    };
    TransformationPtr::new(input_domain, output_domain, function)
}

pub fn make_bounded_sum_ptr<T>(input_domain: &dyn DomainPtr<Carrier=Vec<T>>) -> TransformationPtr<Vec<T>, T> where
    T: 'static + Element + Clone + PartialEq + Sum<T> {
    let input_domain = input_domain.as_any().downcast_ref::<VectorDomainPtr<T>>().expect("Bogus input_domain in make_bounded_sum()");
    let element_domain = &input_domain.element_domain;
    let _element_domain = element_domain.as_any().downcast_ref::<IntervalDomainPtr<T>>().expect("Bogus input_domain in make_bounded_sum()");
    // TODO: Configure stability from bounds of element_domain.
    let input_domain = input_domain.clone();
    let output_domain = AllDomainPtr::<T>::new();
    let function = |arg: &Vec<T>| -> T {
        // FIXME: Can't make this work with references, have to clone.
        let arg = arg.clone();
        arg.into_iter().sum()
    };
    TransformationPtr::new(input_domain, output_domain, function)
}

fn laplace(sigma: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(-0.5, 0.5);
    u.signum() * (1.0 - 2.0 * u.abs()).ln() * sigma
}

pub trait AddNoise {
    fn add_noise(self, noise: f64) -> Self;
}
impl AddNoise for u32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for u64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for i32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for i64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for f32 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for f64 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }
impl AddNoise for u8 { fn add_noise(self, noise: f64) -> Self { (self as f64 + noise) as Self } }

pub fn make_base_laplace_ptr<T>(input_domain: &dyn DomainPtr<Carrier=T>, sigma: f64) -> MeasurementPtr<T, T> where
    T: 'static + Element + Copy + PartialEq + AddNoise {
    let input_domain = input_domain.as_any().downcast_ref::<AllDomainPtr<T>>().expect("Bogus input_domain in make_base_laplace()");
    let input_domain = input_domain.clone();
    let output_domain = AllDomainPtr::<T>::new();
    let function = move |arg: &T| -> T {
        let noise = laplace(sigma);
        arg.add_noise(noise)
    };
    MeasurementPtr::new(input_domain, output_domain, function)
}


mod ffi {
    use std::os::raw::{c_char, c_uint, c_void};

    use crate::core::ffi::{FfiMeasurement, FfiTransformation};
    use crate::ffi_utils;
    use crate::ffi_utils::c_bool;
    use crate::mono::TypeArgs;

    use super::*;

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_identity_ptr(type_args: *const c_char) -> *mut FfiTransformation {
        fn monomorphize<T: 'static + Form + Clone>() -> *mut FfiTransformation {
            let transformation = make_identity_ptr::<T>();
            FfiTransformation::new(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        dispatch!(monomorphize, [(type_args.0[0], @primitives)], ())
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_lines_ptr() -> *mut FfiTransformation {
        let transformation = make_split_lines_ptr();
        FfiTransformation::new(transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_parse_series_ptr(type_args: *const c_char, impute: c_bool) -> *mut FfiTransformation {
        fn monomorphize<T>(impute: bool) -> *mut FfiTransformation where
            T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
            let transformation = make_parse_series_ptr::<T>(impute);
            FfiTransformation::new(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        let impute = ffi_utils::to_bool(impute);
        dispatch!(monomorphize, [(type_args.0[0], @primitives)], (impute))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_records_ptr(separator: *const c_char) -> *mut FfiTransformation {
        let separator = ffi_utils::to_option_str(separator);
        let transformation = make_split_records_ptr(separator);
        FfiTransformation::new(transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_create_dataframe_ptr(col_count: c_uint) -> *mut FfiTransformation {
        let col_count = col_count as usize;
        let transformation = make_create_dataframe_ptr(col_count);
        FfiTransformation::new(transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_dataframe_ptr(separator: *const c_char, col_count: c_uint) -> *mut FfiTransformation {
        let separator = ffi_utils::to_option_str(separator);
        let col_count = col_count as usize;
        let transformation = make_split_dataframe_ptr(separator, col_count);
        FfiTransformation::new(transformation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_parse_column_ptr(type_args: *const c_char, input_transformation: *const FfiTransformation, key: *const c_char, impute: c_bool) -> *mut FfiTransformation {
        fn monomorphize<T>(input_domain: &dyn DomainPtr<Carrier=DataFrame>, key: &str, impute: bool) -> *mut FfiTransformation where
            T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
            let transformation = make_parse_column_ptr::<T>(input_domain, key, impute);
            FfiTransformation::new(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        let input_transformation = ffi_utils::as_ref(input_transformation);
        let input_transformation: &TransformationPtr<(), DataFrame> = input_transformation.as_ref();
        let input_domain = input_transformation.output_domain.as_ref();
        let key = ffi_utils::to_str(key);
        let impute = ffi_utils::to_bool(impute);
        dispatch!(monomorphize, [(type_args.0[0], @primitives)], (input_domain, key, impute))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_select_column_ptr(type_args: *const c_char, input_transformation: *const FfiTransformation, key: *const c_char) -> *mut FfiTransformation {
        fn monomorphize<T>(input_domain: &dyn DomainPtr<Carrier=DataFrame>, key: &str) -> *mut FfiTransformation where
            T: 'static + Element + Clone + PartialEq {
            let transformation = make_select_column_ptr::<T>(input_domain, key);
            FfiTransformation::new(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        let input_transformation = ffi_utils::as_ref(input_transformation);
        let input_transformation: &TransformationPtr<(), DataFrame> = input_transformation.as_ref();
        let input_domain = input_transformation.output_domain.as_ref();
        let key = ffi_utils::to_str(key);
        dispatch!(monomorphize, [(type_args.0[0], @primitives)], (input_domain, key))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_clamp_ptr(type_args: *const c_char, input_transformation: *const FfiTransformation, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation {
        fn monomorphize<T>(input_transformation: &FfiTransformation, lower: *const c_void, upper: *const c_void) -> *mut FfiTransformation where
            T: 'static + Element + Copy + PartialEq + PartialOrd {
            let input_transformation: &TransformationPtr<(), Vec<T>> = input_transformation.as_ref();
            let input_domain = input_transformation.output_domain.as_ref();
            let lower = ffi_utils::as_ref(lower as *const T).clone();
            let upper = ffi_utils::as_ref(upper as *const T).clone();
            let transformation = make_clamp_ptr::<T>(input_domain, lower, upper);
            FfiTransformation::new(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        let input_transformation = ffi_utils::as_ref(input_transformation);
        dispatch!(monomorphize, [(type_args.0[0], @numbers)], (input_transformation, lower, upper))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_bounded_sum_ptr(type_args: *const c_char, input_transformation: *const FfiTransformation) -> *mut FfiTransformation {
        fn monomorphize<T>(input_transformation: &FfiTransformation) -> *mut FfiTransformation where
            T: 'static + Element + Clone + PartialEq + Sum {
            let input_transformation: &TransformationPtr<(), Vec<T>> = input_transformation.as_ref();
            let input_domain = input_transformation.output_domain.as_ref();
            let transformation = make_bounded_sum_ptr::<T>(input_domain);
            FfiTransformation::new(transformation)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        let input_transformation = ffi_utils::as_ref(input_transformation);
        dispatch!(monomorphize, [(type_args.0[0], @numbers)], (input_transformation))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_base_laplace_ptr(type_args: *const c_char, input_transformation: *const FfiTransformation, sigma: f64) -> *mut FfiMeasurement {
        fn monomorphize<T>(input_transformation: &FfiTransformation, sigma: f64) -> *mut FfiMeasurement where
            T: 'static + Element + Copy + PartialEq + AddNoise {
            let input_transformation: &TransformationPtr<(), T> = input_transformation.as_ref();
            let input_domain = input_transformation.output_domain.as_ref();
            let measurement = make_base_laplace_ptr::<T>(input_domain, sigma);
            FfiMeasurement::new(measurement)
        }
        let type_args = TypeArgs::expect(type_args, 1);
        let input_transformation = ffi_utils::as_ref(input_transformation);
        dispatch!(monomorphize, [(type_args.0[0], @numbers)], (input_transformation, sigma))
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__bootstrap() -> *const c_char {
        let spec =
r#"{
    "functions": [
        { "name": "make_identity", "ret": "void *" },
        { "name": "make_split_lines", "ret": "void *" },
        { "name": "make_parse_series", "args": [ ["const char *", "selector"], ["bool", "impute"] ], "ret": "void *" },
        { "name": "make_split_records", "args": [ ["const char *", "separator"] ], "ret": "void *" },
        { "name": "make_create_dataframe", "args": [ ["unsigned int", "col_count"] ], "ret": "void *" },
        { "name": "make_split_dataframe", "args": [ ["const char *", "separator"], ["unsigned int", "col_count"] ], "ret": "void *" },
        { "name": "make_parse_column", "args": [ ["const char *", "selector"], ["const void *", "input_transformation"], ["const char *", "key"], ["bool", "impute"] ], "ret": "void *" },
        { "name": "make_select_column", "args": [ ["const char *", "selector"], ["const void *", "input_transformation"], ["const char *", "key"] ], "ret": "void *" },
        { "name": "make_clamp", "args": [ ["const char *", "selector"], ["const void *", "input_transformation"], ["void *", "lower"], ["void *", "upper"] ], "ret": "void *" },
        { "name": "make_bounded_sum", "args": [ ["const char *", "selector"], ["const void *", "input_transformation"] ], "ret": "void *" },
        { "name": "make_base_laplace", "args": [ ["const char *", "selector"], ["const void *", "input_transformation"], ["double", "sigma"] ], "ret": "void *" }
    ]
}"#;
        ffi_utils::bootstrap(spec)
    }

}


#[cfg(test)]
mod tests {
    use crate::core::make_chain_tt_ptr;

    use super::*;

    #[test]
    fn test_identity_ptr() {
        let identity = make_identity_ptr();
        let arg = 99;
        let ret = identity.invoke(&arg);
        assert_eq!(ret, 99);
    }

    #[test]
    fn test_make_split_lines_ptr() {
        let transformation = make_split_lines_ptr();
        let arg = "ant\nbat\ncat\n".to_owned();
        let ret = transformation.invoke(&arg);
        assert_eq!(ret, vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]);
    }

    #[test]
    fn test_make_parse_series_ptr() {
        let transformation = make_parse_series_ptr::<i32>(true);
        let arg = vec!["1".to_owned(), "2".to_owned(), "3".to_owned(), "foo".to_owned()];
        let ret = transformation.invoke(&arg);
        let expected = vec![1, 2, 3, 0];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_split_records_ptr() {
        let transformation = make_split_records_ptr(None);
        let arg = vec!["ant, foo".to_owned(), "bat, bar".to_owned(), "cat, baz".to_owned()];
        let ret = transformation.invoke(&arg);
        assert_eq!(ret, vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ]);
    }

    #[test]
    fn test_make_create_dataframe_ptr() {
        let transformation = make_create_dataframe_ptr(2);
        let arg = vec![
            vec!["ant".to_owned(), "foo".to_owned()],
            vec!["bat".to_owned(), "bar".to_owned()],
            vec!["cat".to_owned(), "baz".to_owned()],
        ];
        let ret = transformation.invoke(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_split_dataframe_ptr() {
        let transformation = make_split_dataframe_ptr(None, 2);
        let arg = "ant, foo\nbat, bar\ncat, baz".to_owned();
        let ret = transformation.invoke(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_column_ptr() {
        let input_domain = create_raw_dataframe_domain_ptr(2);
        let transformation = make_parse_column_ptr::<i32>(&input_domain, "1", true);
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["1".to_owned(), "2".to_owned(), "".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.invoke(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec![1, 2, 0])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_columns_ptr() {
        let input_domain = create_raw_dataframe_domain_ptr(3);
        let transformation0 = make_parse_column_ptr::<i32>(&input_domain, "1", true);
        let transformation1 = make_parse_column_ptr::<f64>(transformation0.output_domain.as_ref(), "2", true);
        let transformation = make_chain_tt_ptr(&transformation1, &transformation0);
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()])),
            ("2".to_owned(), Data::new(vec!["1.1".to_owned(), "2.2".to_owned(), "3.3".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.invoke(&arg);
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec![1, 2, 3])),
            ("2".to_owned(), Data::new(vec![1.1, 2.2, 3.3])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_select_column_ptr() {
        let input_domain = create_raw_dataframe_domain_ptr(2);
        let transformation = make_select_column_ptr::<String>(&input_domain, "1");
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        let ret = transformation.invoke(&arg);
        let expected = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_clamp_ptr() {
        let input_domain = VectorDomainPtr::<i32>::new_all();
        let transformation = make_clamp_ptr(&input_domain, 0, 10);
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let ret = transformation.invoke(&arg);
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum_ptr() {
        let input_domain = VectorDomainPtr::<i32>::new(IntervalDomainPtr::<i32>::new(Bound::Included(0), Bound::Included(10)));
        let transformation = make_bounded_sum_ptr::<i32>(&input_domain);
        let arg = vec![1, 2, 3, 4, 5];
        let ret = transformation.invoke(&arg);
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_base_laplace_ptr() {
        let input_domain = AllDomainPtr::<f64>::new();
        let measurement = make_base_laplace_ptr::<f64>(&input_domain, 1.0);
        let arg = 0.0;
        let _ret = measurement.invoke(&arg);
        // TODO: Test for base_laplace
    }

}
