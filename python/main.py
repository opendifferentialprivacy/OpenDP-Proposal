import ctypes
import os
import sys

class Domain(ctypes.Structure):
    pass

class Transformation(ctypes.Structure):
    pass

class LibraryWrapper(object):
    def __init__(self):
        # extension = {
        #     "linux": ".so",
        #     "win32": ".dll",
        #     "darwin": ".dylib"
        # }.get(sys.platform)
        #
        # if not extension:
        #     raise Exception(f"opendp does not support {sys.platform}")
        #
        # script_dir = os.path.dirname(os.path.abspath(__file__))
        # lib_dir = os.path.join(script_dir, "lib")
        # lib_opendp_path = os.path.join(lib_dir, "libopendp_proposal" + extension)

        lib_opendp_path = "/Users/michael/OpenDP-Proposal/target/debug/libopendp_proposal.dylib"

        self.lib_opendp = ctypes.cdll.LoadLibrary(lib_opendp_path)

        self.lib_opendp.clamp_f64.argtypes = [ctypes.POINTER(Domain), ctypes.c_double, ctypes.c_double]
        self.lib_opendp.clamp_f64.restype = ctypes.POINTER(Transformation)

        self.lib_opendp.make_default_domain.argtypes = []
        self.lib_opendp.make_default_domain.restype = ctypes.POINTER(Domain)


        # print(self.lib_opendp)

    def make_default_domain(self):
        return self.lib_opendp.make_default_domain()

    def make_clamp(self, input_domain, lower, upper):
        # TODO: wrap lower/upper into enum before calling into lib
        if type(lower) == float and type(upper) == float:
            return self.lib_opendp.clamp_f64(
                input_domain,
                ctypes.c_double(lower),
                ctypes.c_double(upper))



lib_wrapper = LibraryWrapper()

default_domain = lib_wrapper.make_default_domain()
clamp_transform = lib_wrapper.make_clamp(default_domain, 0., 1.)

print(clamp_transform)
