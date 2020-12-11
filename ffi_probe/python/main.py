import ctypes
import json


class Mod:

    name_to_type = {
        "void": None,
        "void *": ctypes.c_void_p,
        "const void *": ctypes.c_void_p,
        "int": ctypes.c_int,
        "int8_t": ctypes.c_int8,
        "int16_t": ctypes.c_int16,
        "int32_t": ctypes.c_int32,
        "int64_t": ctypes.c_int64,
        "unsigned int": ctypes.c_uint,
        "uint8_t": ctypes.c_uint8,
        "uint16_t": ctypes.c_uint16,
        "uint32_t": ctypes.c_uint32,
        "uint64_t": ctypes.c_uint64,
        "float": ctypes.c_float,
        "double": ctypes.c_double,
        "char *": ctypes.c_char_p,
        "const char *": ctypes.c_char_p,
        "bool": ctypes.c_bool,
    }

    @classmethod
    def get_type(cls, name):
        if not name in cls.name_to_type:
            raise Exception(f"Unknown type {name}")
        return cls.name_to_type[name]

    def __init__(self, lib, prefix="ffi__"):
        self.lib = lib
        self.prefix = prefix
        self._bootstrap()

    def _bootstrap(self):
        spec = { "name": "bootstrap", "args": [], "ret": "const char *" }
        _name, bootstrap = self._get_function(spec)
        spec_json = c_char_p_to_str(bootstrap())
        spec = json.loads(spec_json)
        self._load(spec)

    def _load(self, spec):
        for function_spec in spec["functions"]:
            name, function = self._get_function(function_spec)
            self.__setattr__(name, function)

    def _get_function(self, spec):
        name = spec["name"]
        symbol = self.prefix + name
        function = self.lib[symbol]
        function.argtypes = [self.get_type(arg[0]) for arg in spec.get("args", [])]
        function.restype = self.get_type(spec.get("ret", "void"))
        return name, function

class OpenDP:

    def __init__(self, lib):
        self.lib = lib
        self.core = Mod(lib, "opendp_core__")
        self.data = Mod(lib, "opendp_data__")
        self.ops = Mod(lib, "opendp_ops__")


def str_to_c_char_p(s):
    return s.encode("utf-8")

def c_char_p_to_str(s):
    return s.decode("utf-8")

def i32_p(i):
    return ctypes.byref(ctypes.c_int32(i))

def i64_p(i):
    return ctypes.byref(ctypes.c_int64(i))

def f32_p(f):
    return ctypes.byref(ctypes.c_float(f))

def f64_p(f):
    return ctypes.byref(ctypes.c_double(f))

def make_chain(opendp, *operations):
    if not operations:
        raise Exception
    elif len(operations) == 1:
        return operations[0]
    else:
        return make_chain(opendp, *operations[:-2], opendp.core.make_chain(operations[-2], operations[-1]))

def dump(opendp, data):
    string = opendp.data.to_string(data)
    print(c_char_p_to_str(string))


def main():
    lib_path = "../target/debug/libffi_probe.dylib"
    lib = ctypes.cdll.LoadLibrary(lib_path)
    opendp = OpenDP(lib)

    ### HELLO WORLD
    identity = opendp.ops.make_identity(b"<String>")
    arg = opendp.data.from_string(b"hello, world!")
    ret = opendp.core.operation_invoke(identity, arg)
    dump(opendp, ret)
    opendp.data.data_free(arg)
    opendp.data.data_free(ret)
    opendp.core.operation_free(identity)


    ### SUMMARY STATS
    # Parse dataframe
    split_dataframe = opendp.ops.make_split_dataframe(b",", 3)
    parse_column_1 = opendp.ops.make_parse_column(b"<i32>", split_dataframe, b"1", True)
    parse_column_2 = opendp.ops.make_parse_column(b"<f64>", parse_column_1, b"2", True)
    parse_dataframe = make_chain(opendp, parse_column_2, parse_column_1, split_dataframe)

    # Noisy sum, col 1
    select_1 = opendp.ops.make_select_column(b"<i32>", parse_dataframe, b"1")
    clamp_1 = opendp.ops.make_clamp(b"<i32>", select_1, i32_p(0), i32_p(10))
    bounded_sum_1 = opendp.ops.make_bounded_sum(b"<i32>", clamp_1)
    chain_1 = make_chain(opendp, bounded_sum_1, clamp_1, select_1)

    # Noisy sum, col 2
    select_2 = opendp.ops.make_select_column(b"<f64>", parse_dataframe, b"2")
    clamp_2 = opendp.ops.make_clamp(b"<f64>", select_2, f64_p(0.0), f64_p(10.0))
    bounded_sum_2 = opendp.ops.make_bounded_sum(b"<f64>", clamp_2)
    base_laplace_2 = opendp.ops.make_base_laplace(b"<f64>", bounded_sum_2, 1.0)
    chain_2 = make_chain(opendp, base_laplace_2, bounded_sum_2, clamp_2, select_2)

    # Compose & chain
    composition = opendp.core.make_composition(chain_1, chain_2)
    everything = make_chain(opendp, composition, parse_dataframe)

    # Do it!!!
    arg = opendp.data.from_string(b"ant, 1, 1.1\nbat, 2, 2.2\ncat, 3, 3.3")
    ret = opendp.core.operation_invoke(everything, arg)
    dump(opendp, ret)

    # Clean up
    opendp.data.data_free(arg)
    opendp.data.data_free(ret)
    opendp.core.operation_free(everything)

if __name__ == "__main__":
    main()
