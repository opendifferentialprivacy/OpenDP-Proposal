use std::collections::HashMap;
use std::fmt::Debug;
use std::iter;
use std::iter::Sum;
use std::ops::Bound;
use std::str::FromStr;

use rand::Rng;

use crate::core::{Domain, Operation};
use crate::data::{Data, Element, Form};
use crate::dom::{AllDomain, DataDomain, HeterogeneousMapDomain, IntervalDomain, VectorDomain};

pub fn make_identity<T: 'static + Form + Clone>() -> Operation {
    let input_domain = AllDomain::<T>::new();
    let output_domain = AllDomain::<T>::new();
    let function = |arg: &Data| -> Data {
        arg.clone()
    };
    Operation::new(input_domain, output_domain, function)
}

fn clamp<T: Copy + PartialOrd>(lower: T, upper: T, x: &Vec<T>) -> Vec<T> {
    fn clamp1<T: Copy + PartialOrd>(lower: T, upper: T, x: T) -> T {
        if x < lower { lower } else if x > upper { upper } else { x }
    }
    x.into_iter().map(|e| clamp1(lower, upper, *e)).collect()

}

pub fn make_clamp<T>(input_domain: &dyn Domain, lower: T, upper: T) -> Operation where
    T: 'static + Element + Copy + PartialEq + PartialOrd {
    let input_domain = input_domain.as_any().downcast_ref::<VectorDomain<T>>().expect("Bogus input_domain in make_clamp()");
    let input_domain = input_domain.clone();
    let output_domain = VectorDomain::<T>::new(IntervalDomain::new(Bound::Included(lower), Bound::Included(upper)));
    let function = move |arg: &Data| -> Data {
        let arg: &Vec<T> = arg.as_form();
        let ret = clamp(lower, upper, arg);
        Data::new(ret)
    };
    Operation::new(input_domain, output_domain, function)
}

pub fn make_bounded_sum<T>(input_domain: &dyn Domain) -> Operation where
    T: 'static + Element + Clone + PartialEq + Sum<T> {
    let input_domain = input_domain.as_any().downcast_ref::<VectorDomain<T>>().expect("Bogus input_domain in make_bounded_sum()");
    let element_domain = &input_domain.element_domain;
    let _element_domain = element_domain.as_any().downcast_ref::<IntervalDomain<T>>().expect("Bogus input_domain in make_bounded_sum()");
    // TODO: Configure stability from bounds of element_domain.
    let input_domain = input_domain.clone();
    let output_domain = AllDomain::<T>::new();
    let function = |arg: &Data| -> Data {
        let arg: &Vec<T> = arg.as_form();
        // FIXME: Can't make this work with references, have to clone.
        let arg = arg.clone();
        let ret: T = arg.into_iter().sum();
        Data::new(ret)
    };
    Operation::new(input_domain, output_domain, function)
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

pub fn make_split_lines() -> Operation {
    let input_domain = AllDomain::<String>::new();
    let output_domain = VectorDomain::<String>::new_all();
    let function = |arg: &Data| -> Data {
        let arg: &String = arg.as_form();
        let ret = split_lines(arg);
        let ret = vec_str_to_string(ret);
        Data::new(ret)
    };
    Operation::new(input_domain, output_domain, function)
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

pub fn make_parse_series<T>(impute: bool) -> Operation where
    T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
    let input_domain = VectorDomain::<String>::new_all();
    let output_domain = VectorDomain::<T>::new_all();
    let function = move |arg: &Data| -> Data {
        let form: &Vec<String> = arg.as_form();
        let form = vec_string_to_str(form);
        let ret: Vec<T> = parse_series(&form, impute);
        Data::new(ret)
    };
    Operation::new(input_domain, output_domain, function)
}

fn split_records<'a>(separator: &str, lines: &Vec<&'a str>) -> Vec<Vec<&'a str>> {
    fn split<'a>(line: &'a str, separator: &str) -> Vec<&'a str> {
        line.split(separator).into_iter().map(|e| e.trim()).collect()
    }
    lines.into_iter().map(|e| split(e, separator)).collect()
}

pub fn make_split_records(separator: Option<&str>) -> Operation {
    let separator = separator.unwrap_or(",").to_owned();
    let input_domain = VectorDomain::<String>::new_all();
    let output_domain = VectorDomain::<Data>::new(DataDomain::new(VectorDomain::<String>::new_all()));
    let function = move |arg: &Data| -> Data {
        let arg: &Vec<String> = arg.as_form();
        let arg = vec_string_to_str(arg);
        let ret = split_records(&separator, &arg);
        let ret: Vec<Vec<String>> = ret.into_iter().map(vec_str_to_string).collect();
        let ret: Vec<Data> = ret.into_iter().map(Data::new).collect();
        Data::new(ret)
    };
    Operation::new(input_domain, output_domain, function)
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

pub fn create_raw_dataframe_domain(col_count: usize) -> impl Domain {
    let element_domains: HashMap<_, _> = (0..col_count)
        .map(|e|
            (e.to_string(), Box::new(DataDomain::new(VectorDomain::<String>::new_all())) as Box<dyn Domain>))
        .collect();
    HeterogeneousMapDomain::new(element_domains)
}

pub fn make_create_dataframe(col_count: usize) -> Operation {
    let input_domain = VectorDomain::<Data>::new(DataDomain::new(VectorDomain::<String>::new_all()));
    let output_domain = create_raw_dataframe_domain(col_count);
    let function = move |arg: &Data| -> Data {
        let arg: &Vec<Data> = arg.as_form();
        let arg: Vec<_> = arg.into_iter().map(|e| e.as_form::<Vec<String>>()).collect();
        let arg = arg.into_iter().map(|e| vec_string_to_str(e)).collect();
        let ret = create_dataframe(col_count, &arg);
        Data::new(ret)
    };
    Operation::new(input_domain, output_domain, function)
}

fn split_dataframe<'a>(separator: &str, col_count: usize, s: &str) -> DataFrame {
    let lines = split_lines(s);
    let records = split_records(separator, &lines);
    let records = conform_records(col_count, &records);
    create_dataframe(col_count, &records)
}

pub fn make_split_dataframe(separator: Option<&str>, col_count: usize) -> Operation {
    let separator = separator.unwrap_or(",").to_owned();
    let input_domain = AllDomain::<String>::new();
    let output_domain = create_raw_dataframe_domain(col_count);
    let function = move |arg: &Data| -> Data {
        let arg: &String = arg.as_form();
        let ret = split_dataframe(&separator, col_count, &arg);
        Data::new(ret)
    };
    Operation::new(input_domain, output_domain, function)
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

pub fn make_parse_column<T>(input_domain: &dyn Domain, key: &str, impute: bool) -> Operation where
    T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
    let key = key.to_owned();
    let input_domain = input_domain.as_any().downcast_ref::<HeterogeneousMapDomain>().expect("Bogus input_domain in make_parse_column()");
    // TODO: Assert rest of input_domain is valid.
    let input_domain = input_domain.clone();
    let output_element_domains = input_domain.element_domains
        .iter()
        .map(|(k, v)|
            if k == &key { (k.clone(), Box::new(DataDomain::new(VectorDomain::<T>::new_all())) as Box<dyn Domain>) } else { (k.clone(), v.box_clone()) })
        .collect();
    let output_domain = HeterogeneousMapDomain::new(output_element_domains);
    let function = move |arg: &Data| -> Data {
        let arg: &DataFrame = arg.as_form();
        let ret = parse_column::<T>(&key, impute, arg);
        Data::new(ret)
    };
    Operation::new(input_domain, output_domain, function)
}

pub fn make_select_column<T>(input_domain: &dyn Domain, key: &str) -> Operation where
    T: 'static + Element + Clone + PartialEq {
    let key = key.to_owned();
    let input_domain = input_domain.as_any().downcast_ref::<HeterogeneousMapDomain>().expect("Bogus input_domain in make_select_column()");
    let column_domain = input_domain.element_domains.get(&key).expect("Bogus input_domain in make_select_column()");
    let column_domain = column_domain.as_any().downcast_ref::<DataDomain>().expect("Bogus input_domain in make_select_column()");
    // It's a drag that we need a type argument to get column_domain out. Might want to change Operation::new() to take Box<dyn Domain> instead of impl Domain.
    let column_domain = column_domain.form_domain.as_any().downcast_ref::<VectorDomain<T>>().expect("Bogus input_domain in make_select_column()");
    let input_domain = input_domain.clone();
    let output_domain = column_domain.clone();
    let function = move |arg: &Data| -> Data {
        let arg: &DataFrame = arg.as_form();
        let ret = arg.get(&key).expect("Missing dataframe column");
        let ret: &Vec<T> = ret.as_form();
        let ret = ret.clone();
        Data::new(ret)
    };
    Operation::new(input_domain, output_domain, function)
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

pub fn make_base_laplace<T>(input_domain: &dyn Domain, sigma: f64) -> Operation where
    T: 'static + Element + Copy + PartialEq + AddNoise {
    let input_domain = input_domain.as_any().downcast_ref::<AllDomain<T>>().expect("Bogus input_domain in make_base_laplace()");
    let input_domain = input_domain.clone();
    let output_domain = AllDomain::<T>::new();
    let function = move |arg: &Data| -> Data {
        let arg: &T = arg.as_form();
        let noise = laplace(sigma);
        let ret = arg.add_noise(noise);
        Data::new(ret)
    };
    Operation::new(input_domain, output_domain, function)
}


mod ffi {
    use std::convert::TryInto;
    use std::os::raw::{c_char, c_uint, c_void};

    use crate::ffi_utils;
    use crate::ffi_utils::c_bool;
    use crate::mono::Dispatcher;

    use super::*;

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_identity(selector: *const c_char) -> *mut Operation {
        fn monomorphize<T: 'static + Form + Clone>() -> Box<dyn Fn() -> Operation> {
            Box::new(|| make_identity::<T>())
        }
        // TODO: Put dispatcher in lazy_static.
        let mut dispatcher: Dispatcher<Box<_>> = Dispatcher::new();
        register_multi!(dispatcher, monomorphize, [u32, u64, i32, i64, f32, f64, bool, String, u8]);

        let selector = ffi_utils::to_str(selector).try_into().expect("Bogus selector");
        let operation = dispatcher.get(&selector).unwrap()();
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_lines() -> *mut Operation {
        let operation = make_split_lines();
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_parse_series(selector: *const c_char, impute: c_bool) -> *mut Operation {
        fn monomorphize<T>() -> Box<dyn Fn(bool) -> Operation> where
            T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
            Box::new(|impute| make_parse_series::<T>(impute))
        }
        // TODO: Put dispatcher in lazy_static.
        let mut dispatcher: Dispatcher<Box<_>> = Dispatcher::new();
        register_multi!(dispatcher, monomorphize, [u32, u64, i32, i64, f32, f64, bool, u8]);

        let selector = ffi_utils::to_str(selector).try_into().expect("Bogus selector");
        let impute = ffi_utils::to_bool(impute);
        let operation = dispatcher.get(&selector).unwrap()(impute);
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_records(separator: *const c_char) -> *mut Operation {
        let separator = ffi_utils::to_option_str(separator);
        let operation = make_split_records(separator);
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_create_dataframe(col_count: c_uint) -> *mut Operation {
        let col_count = col_count as usize;
        let operation = make_create_dataframe(col_count);
        ffi_utils::into_raw(operation).cast()
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_split_dataframe(separator: *const c_char, col_count: c_uint) -> *mut Operation {
        let separator = ffi_utils::to_option_str(separator);
        let col_count = col_count as usize;
        let operation = make_split_dataframe(separator, col_count);
        ffi_utils::into_raw(operation).cast()
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_parse_column(selector: *const c_char, input_operation: *const Operation, key: *const c_char, impute: c_bool) -> *mut Operation {
        fn monomorphize<T>() -> Box<dyn Fn(&dyn Domain, &str, bool) -> Operation> where
            T: 'static + Element + Clone + PartialEq + FromStr + Default, T::Err: Debug {
            Box::new(|input_domain, key, impute| make_parse_column::<T>(input_domain, key, impute))
        }
        // TODO: Put dispatcher in lazy_static.
        let mut dispatcher: Dispatcher<Box<_>> = Dispatcher::new();
        register_multi!(dispatcher, monomorphize, [u32, u64, i32, i64, f32, f64, bool, u8]);

        let selector = ffi_utils::to_str(selector).try_into().expect("Bogus selector");
        let input_domain = ffi_utils::as_ref(input_operation).output_domain.as_ref();
        let key = ffi_utils::to_str(key);
        let impute = ffi_utils::to_bool(impute);
        let operation = dispatcher.get(&selector).unwrap()(input_domain, key, impute);
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_select_column(selector: *const c_char, input_operation: *const Operation, key: *const c_char) -> *mut Operation {
        fn monomorphize<T>() -> Box<dyn Fn(&dyn Domain, &str) -> Operation> where
            T: 'static + Element + Clone + PartialEq {
            Box::new(|input_domain, key| make_select_column::<T>(input_domain, key))
        }
        // TODO: Put dispatcher in lazy_static.
        let mut dispatcher: Dispatcher<Box<_>> = Dispatcher::new();
        register_multi!(dispatcher, monomorphize, [u32, u64, i32, i64, f32, f64, bool, String, u8]);

        let selector = ffi_utils::to_str(selector).try_into().expect("Bogus selector");
        let input_domain = ffi_utils::as_ref(input_operation).output_domain.as_ref();
        let key = ffi_utils::to_str(key);
        let operation = dispatcher.get(&selector).unwrap()(input_domain, key);
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_clamp(selector: *const c_char, input_operation: *const Operation, lower: *const c_void, upper: *const c_void) -> *mut Operation {
        fn monomorphize<T>() -> Box<dyn Fn(&dyn Domain, *const c_void, *const c_void) -> Operation> where
            T: 'static + Element + Copy + PartialEq + PartialOrd {
            Box::new(|input_domain, lower, upper| {
                let lower = ffi_utils::as_ref(lower as *const T).clone();
                let upper = ffi_utils::as_ref(upper as *const T).clone();
                make_clamp::<T>(input_domain, lower, upper)
            })
        }
        // TODO: Put dispatcher in lazy_static.
        let mut dispatcher: Dispatcher<Box<_>> = Dispatcher::new();
        register_multi!(dispatcher, monomorphize, [u32, u64, i32, i64, f32, f64, bool, u8]);

        let selector = ffi_utils::to_str(selector).try_into().expect("Bogus selector");
        let input_domain = ffi_utils::as_ref(input_operation).output_domain.as_ref();
        let operation = dispatcher.get(&selector).unwrap()(input_domain, lower, upper);
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_bounded_sum(selector: *const c_char, input_operation: *const Operation) -> *mut Operation {
        fn monomorphize<T>() -> Box<dyn Fn(&dyn Domain) -> Operation> where
            T: 'static + Element + Clone + PartialEq + Sum {
            Box::new(|input_domain| make_bounded_sum::<T>(input_domain))
        }
        // TODO: Put dispatcher in lazy_static.
        let mut dispatcher: Dispatcher<Box<_>> = Dispatcher::new();
        register_multi!(dispatcher, monomorphize, [u32, u64, i32, i64, f32, f64, u8]);

        let selector = ffi_utils::to_str(selector).try_into().expect("Bogus selector");
        let input_domain = ffi_utils::as_ref(input_operation).output_domain.as_ref();
        let operation = dispatcher.get(&selector).unwrap()(input_domain);
        ffi_utils::into_raw(operation)
    }

    #[no_mangle]
    pub extern "C" fn opendp_ops__make_base_laplace(selector: *const c_char, input_operation: *const Operation, sigma: f64) -> *mut Operation {
        fn monomorphize<T>() -> Box<dyn Fn(&dyn Domain, f64) -> Operation> where
            T: 'static + Element + Copy + PartialEq + AddNoise {
            Box::new(|input_domain, sigma| {
                make_base_laplace::<T>(input_domain, sigma)
            })
        }
        // TODO: Put dispatcher in lazy_static.
        let mut dispatcher: Dispatcher<Box<_>> = Dispatcher::new();
        register_multi!(dispatcher, monomorphize, [u32, u64, i32, i64, f32, f64]);

        let selector = ffi_utils::to_str(selector).try_into().expect("Bogus selector");
        let input_domain = ffi_utils::as_ref(input_operation).output_domain.as_ref();
        let operation = dispatcher.get(&selector).unwrap()(input_domain, sigma);
        ffi_utils::into_raw(operation)
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
        { "name": "make_parse_column", "args": [ ["const char *", "selector"], ["const void *", "input_operation"], ["const char *", "key"], ["bool", "impute"] ], "ret": "void *" },
        { "name": "make_select_column", "args": [ ["const char *", "selector"], ["const void *", "input_operation"], ["const char *", "key"] ], "ret": "void *" },
        { "name": "make_clamp", "args": [ ["const char *", "selector"], ["const void *", "input_operation"], ["void *", "lower"], ["void *", "upper"] ], "ret": "void *" },
        { "name": "make_bounded_sum", "args": [ ["const char *", "selector"], ["const void *", "input_operation"] ], "ret": "void *" },
        { "name": "make_base_laplace", "args": [ ["const char *", "selector"], ["const void *", "input_operation"], ["double", "sigma"] ], "ret": "void *" }
    ]
}"#;
        ffi_utils::bootstrap(spec)
    }

}


#[cfg(test)]
mod tests {
    use crate::core::make_chain;

    use super::*;

    #[test]
    fn test_make_split_lines() {
        let operation = make_split_lines();
        let arg = "ant\nbat\ncat\n".to_owned();
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: Vec<String> = ret.into_form();
        assert_eq!(ret, vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()]);
    }

    #[test]
    fn test_make_split_records() {
        let operation = make_split_records(None);
        let arg = vec!["ant, foo".to_owned(), "bat, bar".to_owned(), "cat, baz".to_owned()];
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: Vec<Data> = ret.into_form();
        assert_eq!(ret, vec![
            Data::new(vec!["ant".to_owned(), "foo".to_owned()]),
            Data::new(vec!["bat".to_owned(), "bar".to_owned()]),
            Data::new(vec!["cat".to_owned(), "baz".to_owned()]),
        ]);
    }

    #[test]
    fn test_make_create_dataframe() {
        let operation = make_create_dataframe(2);
        let arg = vec![
            Data::new(vec!["ant".to_owned(), "foo".to_owned()]),
            Data::new(vec!["bat".to_owned(), "bar".to_owned()]),
            Data::new(vec!["cat".to_owned(), "baz".to_owned()]),
        ];
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: DataFrame = ret.into_form();
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_split_dataframe() {
        let operation = make_split_dataframe(None, 2);
        let arg = "ant, foo\nbat, bar\ncat, baz".to_owned();
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: DataFrame = ret.into_form();
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_series() {
        let operation = make_parse_series::<i32>(true);
        let arg = vec!["1".to_owned(), "2".to_owned(), "3".to_owned(), "foo".to_owned()];
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: Vec<i32> = ret.into_form();
        let expected = vec![1, 2, 3, 0];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_column() {
        let input_domain = create_raw_dataframe_domain(2);
        let operation = make_parse_column::<i32>(&input_domain, "1", true);
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["1".to_owned(), "2".to_owned(), "".to_owned()])),
        ].into_iter().collect();
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: DataFrame = ret.into_form();
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec![1, 2, 0])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_parse_columns() {
        let input_domain = create_raw_dataframe_domain(3);
        let operation0 = make_parse_column::<i32>(&input_domain, "1", true);
        let operation1 = make_parse_column::<f64>(operation0.output_domain.as_ref(), "2", true);
        let operation = make_chain(operation1, operation0);
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()])),
            ("2".to_owned(), Data::new(vec!["1.1".to_owned(), "2.2".to_owned(), "3.3".to_owned()])),
        ].into_iter().collect();
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: DataFrame = ret.into_form();
        let expected: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec![1, 2, 3])),
            ("2".to_owned(), Data::new(vec![1.1, 2.2, 3.3])),
        ].into_iter().collect();
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_select_column() {
        let input_domain = create_raw_dataframe_domain(2);
        let operation = make_select_column::<String>(&input_domain, "1");
        let arg: DataFrame = vec![
            ("0".to_owned(), Data::new(vec!["ant".to_owned(), "bat".to_owned(), "cat".to_owned()])),
            ("1".to_owned(), Data::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()])),
        ].into_iter().collect();
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: Vec<String> = ret.into_form();
        let expected = vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_clamp() {
        let input_domain = VectorDomain::<i32>::new_all();
        let operation = make_clamp(&input_domain, 0, 10);
        let arg = vec![-10, -5, 0, 5, 10, 20];
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: Vec<i32> = ret.into_form();
        let expected = vec![0, 0, 0, 5, 10, 10];
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_bounded_sum() {
        let input_domain = VectorDomain::<i32>::new(IntervalDomain::<i32>::new(Bound::Included(0), Bound::Included(10)));
        let operation = make_bounded_sum::<i32>(&input_domain);
        let arg = vec![1, 2, 3, 4, 5];
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let ret: i32 = ret.into_form();
        let expected = 15;
        assert_eq!(ret, expected);
    }

    #[test]
    fn test_make_base_laplace() {
        let input_domain = AllDomain::<f64>::new();
        let operation = make_base_laplace::<f64>(&input_domain, 1.0);
        let arg = 0.0;
        let arg = Data::new(arg);
        let ret = operation.invoke(&arg);
        let _ret: f64 = ret.into_form();
        // TODO: Test for base_laplace
    }

}
