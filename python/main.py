import ctypes
import os
import sys
# import numpy as np
# from numpy.ctypeslib import ndpointer

from enum import IntEnum


class Tag(object):
    F64 = 0
    I64 = 1


class Raw(ctypes.Union):
    _fields_ = [("F64", ctypes.c_double),
                ("I64", ctypes.c_int64)]


class Value(ctypes.Structure):
    _fields_ = [("tag", ctypes.c_int32),
                ("data", Raw)]

# class Domain(ctypes.Structure):
#     pass
#
#
# class Transformation(ctypes.Structure):
#     pass


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

        lib_opendp_path = "/Users/michael/OpenDP-Proposal/opendp/target/debug/libopendp.dylib"

        self.lib_opendp = ctypes.cdll.LoadLibrary(lib_opendp_path)

        # self.lib_opendp.clamp_f64.argtypes = [ctypes.POINTER(Domain), ctypes.c_double, ctypes.c_double]
        # self.lib_opendp.clamp_f64.restype = ctypes.POINTER(Transformation)
        #
        # self.lib_opendp.make_default_domain.argtypes = []
        # self.lib_opendp.make_default_domain.restype = ctypes.POINTER(Domain)
        #
        # self.lib_opendp.load_scalar.argtypes = []
        # self.lib_opendp.load_scalar.restype = ctypes.POINTER(Domain)
        #
        # _doublepp = ndpointer(dtype=np.uintp, ndim=1, flags='C')
        # self.lib_opendp.release.argtypes = (_doublepp, ctypes.c_int, ctypes.c_int)
        # self.lib_opendp.release.restype = ctypes.c_char_p

    def make_default_domain(self):
        return self.lib_opendp.make_default_domain()

    # def make_clamp(self, input_domain, lower, upper):
    #     # TODO: wrap lower/upper into enum before calling into lib
    #     if type(lower) == float and type(upper) == float:
    #         return self.lib_opendp.clamp_f64(
    #             input_domain,
    #             ctypes.c_double(lower),
    #             ctypes.c_double(upper))
    #
    # def load_scalar(self, data):
    #     if type(data) == float:
    #         rust_annotated_data = self.lib_opendp.build_f64(ctypes.c_double(data))
    #         self.lib_opendp.load_any(rust_annotated_data)
    #
    # def pass_vec(self, data):
    #     self.lib_opendp.load_f64_vec(
    #
    #     )
    #
    # def pass_array(self, data):
    #     self.lib_opendp.load_f64_array(
    #         (data.__array_interface__['data'][0] + np.arange(data.shape[0]) * data.strides[0]).astype(np.uintp),
    #         ctypes.c_int(data.shape[0]),
    #         ctypes.c_int(data.shape[1]))

    def load_scalar(self, data):

        if type(data) == float:
            value = Value(Tag.F64, Raw(F64=data))
        elif type(data) == int:
            value = Value(Tag.I64, Raw(I64=data))
        else:
            raise ValueError(f"unrecognized type: {type(data)}")
        self.lib_opendp.load_scalar(ctypes.byref(value))


lib_wrapper = LibraryWrapper()

# default_domain = lib_wrapper.make_default_domain()
# clamp_transform = lib_wrapper.make_clamp(default_domain, 0., 1.)

# I can parse this as a struct in python ctypes, and modify the domain
# clamp_transform
#
# print(clamp_transform)

lib_wrapper.load_scalar(22.)
lib_wrapper.load_scalar(22)