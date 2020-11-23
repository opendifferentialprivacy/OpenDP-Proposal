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

def dump(opendp, data):
    string = opendp.data.to_string(data)
    print(c_char_p_to_str(string))

def main():
    lib_path = "../target/debug/libffi_probe.dylib"
    lib = ctypes.cdll.LoadLibrary(lib_path)
    opendp = OpenDP(lib)

    identity = opendp.ops.make_identity()
    arg = opendp.data.from_string(str_to_c_char_p("hello, world!"))
    ret = opendp.core.operation_invoke(identity, arg)
    dump(opendp, ret)
    # opendp.data.data_free(arg)
    opendp.data.data_free(ret)
    opendp.core.operation_free(identity)

    parse_dataframe = opendp.ops.make_parse_dataframe(str_to_c_char_p(","), True)
    arg = opendp.data.from_string(str_to_c_char_p("ant, 1, 1.1\nbat, 2, 2.2\ncat, 3, 3.3"))
    ret = opendp.core.operation_invoke(parse_dataframe, arg)
    dump(opendp, ret)
    # opendp.data.data_free(arg)
    opendp.data.data_free(ret)
    opendp.core.operation_free(identity)

if __name__ == "__main__":
    main()
