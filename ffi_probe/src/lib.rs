use std::any::Any;
use std::ffi::c_void;
use std::fmt::Debug;


//////////////////////////////////////////////////
// DEFINITION OF OPERATIONS
//////////////////////////////////////////////////

#[repr(C)]
pub struct Operation<T, U> {
    pub function: Box<dyn Fn(T) -> U>
}

pub fn make_clamp<T: 'static + PartialOrd + Copy + Debug>(min: T, max: T) -> Operation<T, T> {
    println!("make_clamp({:?}, {:?})", min, max);
    Operation {
        function: Box::new(move |x: T| {
            let res = if x > max { max } else { if x < min { min } else { x } };
            println!("clamp_{:?}_{:?}({:?}) => {:?}", min, max, x, res);
            res
        })
    }
}

pub struct Heterogeneous<X, Y> {
    pub x: X,
    pub y: Y,
}

pub fn make_heterogeneous<T, U>() -> Operation<Heterogeneous<T, U>, f64> {
    Operation {
        function: Box::new(move |_x: Heterogeneous<T, U>| {
            0.999
        })
    }
}



#[test]
fn test_simple() {
    let clamp_f64 = make_clamp(0.0, 10.0);
    assert_eq!(5.0, (clamp_f64.function)(5.0));
    assert_eq!(0.0, (clamp_f64.function)(-5.0));
    assert_eq!(10.0, (clamp_f64.function)(15.0));

    let clamp_i64 = make_clamp(0, 10);
    assert_eq!(5, (clamp_i64.function)(5));
    assert_eq!(0, (clamp_i64.function)(-5));
    assert_eq!(10, (clamp_i64.function)(15));
}


//////////////////////////////////////////////////
// COMMON STUFF FOR TAGS & UNIONS
//////////////////////////////////////////////////

#[repr(C)]
pub enum Tag {
    I32,
    I64,
    F32,
    F64,
}

#[repr(C)]
pub union Value {
    i32: i32,
    i64: i64,
    f32: f32,
    f64: f64,
}


//////////////////////////////////////////////////
// COMMON STUFF FOR TRAIT OBJECTS
//////////////////////////////////////////////////

trait FFIObject {
    fn as_any(&self) -> &dyn Any;
}
impl FFIObject for Operation<i32, i32> {
    fn as_any(&self) -> &dyn Any { self }
}
impl FFIObject for Operation<i64, i64> {
    fn as_any(&self) -> &dyn Any { self }
}
impl FFIObject for Operation<f32, f32> {
    fn as_any(&self) -> &dyn Any { self }
}
impl FFIObject for Operation<f64, f64> {
    fn as_any(&self) -> &dyn Any { self }
}



//////////////////////////////////////////////////
// OPTION 1-A:
//   * Represent operations as plain structs.
//   * Dispatch by monomorphized functions at the FFI layer.
//   * Pass values directly.
//
// make_clamp = get_ffi("make_clamp_struct_f64", [ctypes.c_double, ctypes.c_double], ctypes.POINTER(Operation))
// invoke = get_ffi("invoke_struct_f64", [ctypes.POINTER(Operation), ctypes.c_double], ctypes.c_double)
// clamp = make_clamp(5.0, 10.0)
// res = invoke(clamp, -1.0)
// print("res = {}".format(res))
//////////////////////////////////////////////////

#[no_mangle]
pub extern "C" fn make_clamp_struct_i32(min: i32, max: i32) -> *const Operation<i32, i32> {
    Box::into_raw(Box::new(make_clamp(min, max)))
}
#[no_mangle]
pub extern "C" fn make_clamp_struct_i64(min: i64, max: i64) -> *const Operation<i64, i64> {
    Box::into_raw(Box::new(make_clamp(min, max)))
}
#[no_mangle]
pub extern "C" fn make_clamp_struct_f32(min: f32, max: f32) -> *const Operation<f32, f32> {
    Box::into_raw(Box::new(make_clamp(min, max)))
}
#[no_mangle]
pub extern "C" fn make_clamp_struct_f64(min: f64, max: f64) -> *const Operation<f64, f64> {
    Box::into_raw(Box::new(make_clamp(min, max)))
}

#[no_mangle]
pub unsafe extern "C" fn invoke_struct_i32(operation: *const Operation<i32, i32>, x: i32) -> i32 {
    ((*operation).function)(x)
}
#[no_mangle]
pub unsafe extern "C" fn invoke_struct_i64(operation: *const Operation<i64, i64>, x: i64) -> i64 {
    ((*operation).function)(x)
}
#[no_mangle]
pub unsafe extern "C" fn invoke_struct_f32(operation: *const Operation<f32, f32>, x: f32) -> f32 {
    ((*operation).function)(x)
}
#[no_mangle]
pub unsafe extern "C" fn invoke_struct_f64(operation: *const Operation<f64, f64>, x: f64) -> f64 {
    ((*operation).function)(x)
}



//////////////////////////////////////////////////
// OPTION 1-B-1:
//   * Represent operations as plain structs.
//   * Dispatch by tag.
//   * Pass values in unions.
//
// make_clamp = get_ffi("make_clamp_struct_tag_union", [Tag, Value, Value], ctypes.POINTER(Operation))
// invoke = get_ffi("invoke_struct_tag_union", [Tag, ctypes.POINTER(Operation), Value], Value)
// clamp = make_clamp(Tag_F64, Value(f64=5.0), Value(f64=10.0))
// res = invoke(Tag_F64, clamp, Value(f64=-1.0))
// print("res = {}".format(res.f64))
//////////////////////////////////////////////////

#[no_mangle]
pub unsafe extern "C" fn make_clamp_struct_tag_union(tag: Tag, min: Value, max: Value) -> *const c_void {
    match tag {
        Tag::I32 => { Box::into_raw(Box::new(make_clamp(min.i32, max.i32))) as *const c_void }
        Tag::I64 => { Box::into_raw(Box::new(make_clamp(min.i64, max.i64))) as *const c_void }
        Tag::F32 => { Box::into_raw(Box::new(make_clamp(min.f32, max.f32))) as *const c_void }
        Tag::F64 => { Box::into_raw(Box::new(make_clamp(min.f64, max.f64))) as *const c_void }
    }
}

#[no_mangle]
pub unsafe extern "C" fn invoke_struct_tag_union(tag: Tag, operation: *const c_void, x: Value) -> Value {
    match tag {
        Tag::I32 => { Value { i32: ((*(operation as *const Operation<i32, i32>)).function)(x.i32) } }
        Tag::I64 => { Value { i64: ((*(operation as *const Operation<i64, i64>)).function)(x.i64) } }
        Tag::F32 => { Value { f32: ((*(operation as *const Operation<f32, f32>)).function)(x.f32) } }
        Tag::F64 => { Value { f64: ((*(operation as *const Operation<f64, f64>)).function)(x.f64) } }
    }
}


//////////////////////////////////////////////////
// OPTION 1-B-2:
//   * Represent operations as plain structs.
//   * Dispatch by tag.
//   * Pass values with pointers.
//
// make_clamp = get_ffi("make_clamp_struct_tag_pointer", [Tag, ctypes.c_void_p, ctypes.c_void_p], ctypes.POINTER(Operation))
// invoke = get_ffi("invoke_struct_tag_pointer", [Tag, ctypes.POINTER(Operation), ctypes.c_void_p], ctypes.c_void_p)
// clamp = make_clamp(Tag_F64, ctypes.byref(ctypes.c_double(5.0)), ctypes.byref(ctypes.c_double(10.0)))
// res = invoke(Tag_F64, clamp, ctypes.byref(ctypes.c_double(-1.0)))
// print("res = {}".format(ctypes.cast(res, ctypes.POINTER(ctypes.c_double)).contents))
//////////////////////////////////////////////////

#[no_mangle]
pub unsafe extern "C" fn make_clamp_struct_tag_pointer(tag: Tag, min: *const c_void, max: *const c_void) -> *const c_void {
    match tag {
        Tag::I32 => { Box::into_raw(Box::new(make_clamp(*(min as *const i32), *(max as *const i32)))) as *const c_void }
        Tag::I64 => { Box::into_raw(Box::new(make_clamp(*(min as *const i64), *(max as *const i64)))) as *const c_void }
        Tag::F32 => { Box::into_raw(Box::new(make_clamp(*(min as *const f32), *(max as *const f32)))) as *const c_void }
        Tag::F64 => { Box::into_raw(Box::new(make_clamp(*(min as *const f64), *(max as *const f64)))) as *const c_void }
    }
}

#[no_mangle]
pub unsafe extern "C" fn invoke_struct_tag_pointer(tag: Tag, operation: *const c_void, x: *const c_void) -> *const c_void {
    match tag {
        Tag::I32 => { Box::into_raw(Box::new(((*(operation as *const Operation<i32, i32>)).function)(*(x as *const i32)))) as *const c_void }
        Tag::I64 => { Box::into_raw(Box::new(((*(operation as *const Operation<i64, i64>)).function)(*(x as *const i64)))) as *const c_void }
        Tag::F32 => { Box::into_raw(Box::new(((*(operation as *const Operation<f32, f32>)).function)(*(x as *const f32)))) as *const c_void }
        Tag::F64 => { Box::into_raw(Box::new(((*(operation as *const Operation<f64, f64>)).function)(*(x as *const f64)))) as *const c_void }
    }
}


//////////////////////////////////////////////////
// OPTION 2-A:
//   * Represent operations as trait objects.
//   * Dispatch by monomorphized functions at the FFI layer (constructors).
//   * Dispatch by trait (invokers).
//   * Pass values directly (constructors).
//   * Pass values in unions (invokers).
//
// make_clamp = get_ffi("make_clamp_trait_f64", [ctypes.c_double, ctypes.c_double], ctypes.c_void_p)
// invoke = get_ffi("invoke_trait_union", [ctypes.c_void_p, Value], Value)
// clamp = make_clamp(5.0, 10.0)
// res = invoke(clamp, Value(f64=-1.0))
// print("res = {}".format(res.f64))
//////////////////////////////////////////////////

#[no_mangle]
pub extern "C" fn make_clamp_trait_i32(min: i32, max: i32) -> *const c_void {
    Box::into_raw(Box::new(Box::new(make_clamp(min, max)) as Box<dyn FFIObject>)) as *const c_void
}
#[no_mangle]
pub extern "C" fn make_clamp_trait_i64(min: i64, max: i64) -> *const c_void {
    Box::into_raw(Box::new(Box::new(make_clamp(min, max)) as Box<dyn FFIObject>)) as *const c_void
}
#[no_mangle]
pub extern "C" fn make_clamp_trait_f32(min: f32, max: f32) -> *const c_void {
    Box::into_raw(Box::new(Box::new(make_clamp(min, max)) as Box<dyn FFIObject>)) as *const c_void
}
#[no_mangle]
pub extern "C" fn make_clamp_trait_f64(min: f64, max: f64) -> *const c_void {
    Box::into_raw(Box::new(Box::new(make_clamp(min, max)) as Box<dyn FFIObject>)) as *const c_void
}

#[no_mangle]
pub unsafe extern "C" fn invoke_trait_union(operation: *const c_void, x: Value) -> Value {
    let operation = (&*(operation as *const Box<dyn FFIObject>)).as_any();
    if let Some(operation) = operation.downcast_ref::<Operation<i32, i32>>() {
        Value { i32: (operation.function)(x.i32) }
    } else if let Some(operation) = operation.downcast_ref::<Operation<i64, i64>>() {
        Value { i64: (operation.function)(x.i64) }
    } else if let Some(operation) = operation.downcast_ref::<Operation<f32, f32>>() {
        Value { f32: (operation.function)(x.f32) }
    } else if let Some(operation) = operation.downcast_ref::<Operation<f64, f64>>() {
        Value { f64: (operation.function)(x.f64) }
    } else {
        panic!("CRAP")
    }
}

//////////////////////////////////////////////////
// OPTION 2-B-1:
//   * Represent operations as trait objects.
//   * Dispatch by tag (constructors).
//   * Dispatch by trait (invokers).
//   * Pass values in unions.
//
// make_clamp = get_ffi("make_clamp_trait_tag_union", [Tag, Value, Value], ctypes.c_void_p)
// invoke = get_ffi("invoke_trait_union", [ctypes.c_void_p, Value], Value)
// clamp = make_clamp(Tag_F64, Value(f64=5.0), Value(f64=10.0))
// res = invoke(clamp, Value(f64=-1.0))
// print("res = {}".format(res.f64))
//////////////////////////////////////////////////

#[no_mangle]
pub unsafe extern "C" fn make_clamp_trait_tag_union(tag: Tag, min: Value, max: Value) -> *const c_void {
    match tag {
        Tag::I32 => { Box::into_raw(Box::new(Box::new(make_clamp(min.i32, max.i32)) as Box<dyn FFIObject>)) as *const c_void }
        Tag::I64 => { Box::into_raw(Box::new(Box::new(make_clamp(min.i64, max.i64)) as Box<dyn FFIObject>)) as *const c_void }
        Tag::F32 => { Box::into_raw(Box::new(Box::new(make_clamp(min.f32, max.f32)) as Box<dyn FFIObject>)) as *const c_void }
        Tag::F64 => { Box::into_raw(Box::new(Box::new(make_clamp(min.f64, max.f64)) as Box<dyn FFIObject>)) as *const c_void }
    }
}

// INVOKE IS SAME AS OPTION 2-A


//////////////////////////////////////////////////
// OPTION 2-B-2:
//   * Represent operations as trait objects.
//   * Dispatch by tag (constructors).
//   * Dispatch by trait (invokers).
//   * Pass values as pointer.
// EXERCISE FOR THE READER!!!
//////////////////////////////////////////////////
