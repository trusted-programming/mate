# cargo-mate

There are 3 sections -- `rules`, `safety`, and `parallel`: 
- the goal of `rules` is to make all the rules *machine applicable*;
- the goal of `safety` is to detect opportunities to convert `unsafe` code into safe ones;
- the goal of `parallel` is to lint all the places where there is a possibility to parallelize the code. 

### installation

1. clone the project
    ```
    git clone git@github.com:trusted-programming/cargo-mate.git
    cd cargo-mate
    ```
2. cargo install it 
    ```
    cargo install --path .
    ```
3. copy libs files in your toolchain cargo-mate is dynamically linked to 2 rust nightly libs, before the tool can work it need
    ```
    cp ~/.rustup/toolchains/<TOOLCHAIN nightly-2023-08-24>/lib/librustc_driver-df8e10c587cf8daa.dylib ~/.rustup/toolchains/<TOOLCHAIN of project>/lib/
    cp ~/.rustup/toolchains/<TOOLCHAIN nightly-2023-08-24>/lib/libstd-bff7f270c7778e6c.dylib ~/.rustup/toolchains/<TOOLCHAIN of project>/lib/

    Example for apple silicon Mac on project running on the stable toolchain:
    cp ~/.rustup/toolchains/nightly-2023-08-24-aarch64-apple-darwin/lib/librustc_driver-df8e10c587cf8daa.dylib ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/
    cp ~/.rustup/toolchains/nightly-2023-08-24-aarch64-apple-darwin/lib/libstd-bff7f270c7778e6c.dylib ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/
    ```
4. run `cargo mate` by default it will run the parallel suggestions

### update to latest
1. clone the project
    ```
    git clone git@github.com:trusted-programming/cargo-mate.git
    or
    cd cargo-mate
    git pull
    ```
2. cargo install it using force to overwrite previous
    ```
    cargo install --path . --force
    ```
3. if libs are missing copy in your toolchain


### benchmarks
-  https://github.com/alexcrichton/tar-rs
-  https://gitee.com/openharmony/commonlibrary_rust_ylong_runtime
-  https://gitee.com/organizations/openharmony/projects?lang=Rust


## Parallel list

## Rules list
1. non_snake_case
2. non_upper_case_globals
3. non_camel_case_types 
4. missing_debug_implementations 
5. clippy::wrong_self_convention 
6. clippy::missing_errors_doc 
7. clippy::missing_panics_doc
8. clippy::undocumented_unsafe_blocks
9. clippy::missing_safety_doc 
10. clippy::approx_constant 
11. clippy::borrow_interior_mutable_const 
12. clippy::declare_interior_mutable_const 
13. clippy::default_numeric_fallback 
14. clippy::bool_assert_comparison 
15. clippy::bool_comparison 
16. clippy::blocks_in_if_conditions 
17. clippy::arithmetic_side_effects 
18. clippy::needless_range_loop 
19. clippy::recursive_format_impl 
20. clippy::precedence clippy::bad_bit_mask 
21. clippy::match_overlapping_arm 
22. clippy::large_types_passed_by_value 
23. clippy::derived_hash_with_manual_eq 
24. clippy::derive_ord_xor_partial_ord 
25. clippy::from_over_into 
26. clippy::unwrap_used 
27. clippy::expect_used 
28. clippy::wildcard_imports 
29. clippy::mut_from_ref 
30. clippy::mutex_atomic 
31. clippy::mutex_integer 
32. clippy::redundant_allocation

## Unsafe to Safe list



