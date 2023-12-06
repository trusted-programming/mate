# cargo-mate


## how to run

The next three steps install Dylint and run all of this repository's lints on a workspace:

1. Install `cargo-dylint` and `dylint-link`:

   ```sh
   cargo install cargo-dylint dylint-link
   ```

2. Add the following to the workspace's `Cargo.toml` file:

   ```toml
   [workspace.metadata.dylint]
   libraries = [
       { git = "https://github.com/trusted-programming/cargo-mate"},
   ]
   ```

3. Run `cargo-dylint` for linting:
   ```sh
    # lint only
    cargo dylint --all --workspace
    # lint and fix, if code breaks(not compiling) it will revert to original
    cargo dylint --all --workspace --fix
    # lint and fix ignoring errors if there are any
    cargo dylint --all --workspace --fix -- --broken-code
    ```

In the above example, the libraries are found via [workspace metadata], which is the recommended way. For additional ways of finding libraries, see [How Dylint works].


### benchmarks
-  https://github.com/alexcrichton/tar-rs
-  https://gitee.com/openharmony/commonlibrary_rust_ylong_runtime
-  https://gitee.com/organizations/openharmony/projects?lang=Rust


