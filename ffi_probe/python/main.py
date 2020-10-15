import ctypes


lib_path = "../target/debug/libffi_probe.dylib"
lib = ctypes.cdll.LoadLibrary(lib_path)

def get_ffi(name, argtypes, restype):
    fn = lib[name]
    fn.argtypes = argtypes
    fn.restype = restype
    return fn


class Operation(ctypes.Structure):
    pass

Tag = ctypes.c_int
Tag_I32 = 0
Tag_I64 = 1
Tag_F32 = 2
Tag_F64 = 3

class Value(ctypes.Union):
    _fields_ = [("i32", ctypes.c_int32), ("i64", ctypes.c_int64), ("f32", ctypes.c_float), ("f64", ctypes.c_double)]


print("\nOPTION 1-A")
make_clamp = get_ffi("make_clamp_struct_f64", [ctypes.c_double, ctypes.c_double], ctypes.POINTER(Operation))
invoke = get_ffi("invoke_struct_f64", [ctypes.POINTER(Operation), ctypes.c_double], ctypes.c_double)
clamp = make_clamp(5.0, 10.0)
res = invoke(clamp, -1.0)
print("res = {}".format(res))


print("\nOPTION 1-B-1")
make_clamp = get_ffi("make_clamp_struct_tag_union", [Tag, Value, Value], ctypes.POINTER(Operation))
invoke = get_ffi("invoke_struct_tag_union", [Tag, ctypes.POINTER(Operation), Value], Value)
clamp = make_clamp(Tag_F64, Value(f64=5.0), Value(f64=10.0))
res = invoke(Tag_F64, clamp, Value(f64=-1.0))
print("res = {}".format(res.f64))


print("\nOPTION 1-B-2")
make_clamp = get_ffi("make_clamp_struct_tag_pointer", [Tag, ctypes.c_void_p, ctypes.c_void_p], ctypes.POINTER(Operation))
invoke = get_ffi("invoke_struct_tag_pointer", [Tag, ctypes.POINTER(Operation), ctypes.c_void_p], ctypes.c_void_p)
clamp = make_clamp(Tag_F64, ctypes.byref(ctypes.c_double(5.0)), ctypes.byref(ctypes.c_double(10.0)))
res = invoke(Tag_F64, clamp, ctypes.byref(ctypes.c_double(-1.0)))
print("res = {}".format(ctypes.cast(res, ctypes.POINTER(ctypes.c_double)).contents))


print("\nOPTION 2-A")
make_clamp = get_ffi("make_clamp_trait_f64", [ctypes.c_double, ctypes.c_double], ctypes.c_void_p)
invoke = get_ffi("invoke_trait_union", [ctypes.c_void_p, Value], Value)
clamp = make_clamp(5.0, 10.0)
res = invoke(clamp, Value(f64=-1.0))
print("res = {}".format(res.f64))


print("\nOPTION 2-B-1")
make_clamp = get_ffi("make_clamp_trait_tag_union", [Tag, Value, Value], ctypes.c_void_p)
invoke = get_ffi("invoke_trait_union", [ctypes.c_void_p, Value], Value)
clamp = make_clamp(Tag_F64, Value(f64=5.0), Value(f64=10.0))
res = invoke(clamp, Value(f64=-1.0))
print("res = {}".format(res.f64))


print("\nIT DIDN'T CRASH!!!")
