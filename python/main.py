import ctypes
import os
import sys


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

        self.lib_opendp.clamp_f64.argtypes = [ctypes.c_double, ctypes.c_double]
        # self.lib_opendp.clamp_f64.restype =

        # print(self.lib_opendp)

    def make_clamp(self, lower, upper):
        # TODO: wrap lower/upper into enum before calling into lib
        # TODO: resolve memory mapping issues on return
        if type(lower) == float and type(upper) == float:
            return self.lib_opendp.clamp_f64(
                ctypes.c_double(lower),
                ctypes.c_double(upper))



lib_wrapper = LibraryWrapper()


clamp_transform = lib_wrapper.make_clamp(0., 1.)
print(clamp_transform)
