import opendp

def main():
    lib_path = "../target/debug/libffi_probe.dylib"
    odp = opendp.OpenDP(lib_path)

    ### HELLO WORLD
    identity = odp.ops.make_identity(b"<String>")
    arg = odp.data.from_string(b"hello, world!")
    ret = odp.core.transformation_invoke(identity, arg)
    print(odp.to_str(ret))
    odp.data.data_free(arg)
    odp.data.data_free(ret)
    odp.core.transformation_free(identity)

    ### SUMMARY STATS
    # Parse dataframe
    split_dataframe = odp.ops.make_split_dataframe(b",", 3)
    parse_column_1 = odp.ops.make_parse_column(b"<i32>", split_dataframe, b"1", True)
    parse_column_2 = odp.ops.make_parse_column(b"<f64>", parse_column_1, b"2", True)
    parse_dataframe = odp.make_chain_tt_multi(parse_column_2, parse_column_1, split_dataframe)

    # Noisy sum, col 1
    select_1 = odp.ops.make_select_column(b"<i32>", parse_dataframe, b"1")
    clamp_1 = odp.ops.make_clamp(b"<i32>", select_1, odp.i32_p(0), odp.i32_p(10))
    bounded_sum_1 = odp.ops.make_bounded_sum(b"<i32>", clamp_1, odp.i32_p(0), odp.i32_p(10))
    base_laplace_1 = odp.ops.make_base_laplace(b"<i32>", bounded_sum_1, 1.0)
    noisy_sum_1 = odp.core.make_chain_mt(base_laplace_1, odp.make_chain_tt_multi(bounded_sum_1, clamp_1, select_1))

    # Noisy sum, col 2
    select_2 = odp.ops.make_select_column(b"<f64>", parse_dataframe, b"2")
    clamp_2 = odp.ops.make_clamp(b"<f64>", select_2, odp.f64_p(0.0), odp.f64_p(10.0))
    bounded_sum_2 = odp.ops.make_bounded_sum(b"<f64>", clamp_2, odp.f64_p(0.0), odp.f64_p(10.0))
    base_laplace_2 = odp.ops.make_base_laplace(b"<f64>", bounded_sum_2, 1.0)
    noisy_sum_2 = odp.core.make_chain_mt(base_laplace_2, odp.make_chain_tt_multi(bounded_sum_2, clamp_2, select_2))

    # Compose & chain
    composition = odp.core.make_composition(noisy_sum_1, noisy_sum_2)
    everything = odp.core.make_chain_tt(composition, parse_dataframe)

    # Do it!!!
    arg = odp.data.from_string(b"ant, 1, 1.1\nbat, 2, 2.2\ncat, 3, 3.3")
    ret = odp.core.measurement_invoke(everything, arg)
    print(odp.to_str(ret))

    # Clean up
    odp.data.data_free(arg)
    odp.data.data_free(ret)
    odp.core.measurement_free(everything)

if __name__ == "__main__":
    main()
