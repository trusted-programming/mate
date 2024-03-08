# Mate

Mate is a linting library for the Rust programming language with a focus on identifying opportunities for parallelization in code. It aims to help developers leverage the power of parallel computing by providing automated suggestions for optimizing their Rust applications.

## Features

- **Automatic Detection**: Mate scans your codebase to find potential areas where parallelization can be applied.
- **Optimization Suggestions**: Provides recommendations on how to modify your code to take advantage of parallel execution.
- **Easy Integration**: Seamlessly integrates with your existing Rust projects and tooling.
- **Automated Suggestions Application**: Mate can automatically implement its parallelization recommendations in your code through rustfix, streamlining the optimization process.

## Lints

- for_each
- filter_simple
- filter_simple_flipped
- fold_simple
- fold_vec
- fold_hashmap
- par_fold_simple
- par_fold_vec
- rayon_prelude
- par_iter

## Warnings

often switching from an iterator to a parallel iterator will result in loss of ordering:

```rust
// will print in order from 0 to 100
(0..100).into_iter().for_each(|x| println!("{:?}", x)); // 0 1 2 3 ...
// will print in a random order depending on threads
(0..100).into_par_iter().for_each(|x| println!("{:?}", x)); // 56 87 37 88 ...
```

## How to run

The next three steps install Dylint and run all of this repository's lints on a workspace:

prerequisites: - rustup - latest version of rust stable if not run `rustup update`

1. Install `cargo-dylint` and `dylint-link`:

   ```sh
   cargo install cargo-dylint dylint-link
   ```

2. Add the following to the workspace's `Cargo.toml` file:

   ```toml
   [workspace.metadata.dylint]
   libraries = [
       { git = "https://github.com/trusted-programming/mate"},
   ]
   ```

3. Run `cargo-dylint` for linting:
   ```sh
    # lint only
    cargo dylint --all --workspace
    # lint and fix, if code breaks(not compiling) it will revert to original
    cargo dylint --all --workspace --fix -- --allow-dirty --allow-no-vcs
    # lint and fix ignoring errors if there are any
    cargo dylint --all --workspace --fix -- --allow-dirty --allow-no-vcs --broken-code
    # count warnings by type
    cargo dylint --workspace --all 2>&1 | grep -i 'warning' | grep -iv 'generated' | sort | uniq -c | sort -nr
   ```

In the above example, the libraries are found via [workspace metadata], which is the recommended way. For additional ways of finding libraries, see [How Dylint works].

## Running Tests

To execute tests in a Rust project, use the following commands:

```sh
# Run all tests in the workspace
cargo test --workspace

# Run tests in a specific nested crate
cargo test -p <nested_crate_name>
```

Replace `<nested_crate_name>` with the name of the nested crate for which you want to run the tests.

For more information on testing in Rust, refer to the [Rust Book's section on testing](https://doc.rust-lang.org/book/ch11-00-testing.html).

### VS Code integration

Dylint results can be viewed in VS Code using [rust-analyzer]. To do so, add the following to your VS Code `settings.json` file:

```json
    "rust-analyzer.checkOnSave.overrideCommand": [
        "cargo",
        "dylint",
        "--all",
        "--workspace",
        "--",
        "--all-targets",
        "--message-format=json"
    ]
```

If you want to use rust-analyzer inside a lint library, you need to add the following to your VS Code `settings.json` file:

```json
    "rust-analyzer.rustc.source": "discover",
```

And add the following to the library's `Cargo.toml` file:

```toml
[package.metadata.rust-analyzer]
rustc_private = true
```

## benchmarks

- https://gitee.com/openharmony/request_request
- https://gitee.com/openharmony/commonlibrary_rust_ylong_runtime
- https://gitee.com/organizations/openharmony/projects?lang=Rust
