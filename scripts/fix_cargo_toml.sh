#!/bin/bash

# Find all Cargo.toml files and process them for rayon dependency
find . -type f -name "Cargo.toml" | while read -r file; do
    # Check if rayon is already in the dependencies
    if ! grep -qE "^rayon =" "$file"; then
        # Add rayon under the [dependencies] line
        sed -i '/\[dependencies\]/a rayon = "1.8.1"' "$file"
    fi
done

# Process only 1 layer deep Cargo.toml files for dylint workspace metadata
find . -maxdepth 2 -type f -name "Cargo.toml" | while read -r file; do
    # Append dylint workspace metadata at the end of the file if not present
    if ! grep -qE "\[workspace.metadata.dylint\]" "$file"; then
        echo -e "\n[workspace.metadata.dylint]\nlibraries = [\n    { git = \"https://github.com/trusted-programming/cargo-mate\"},\n]" >>"$file"
    fi
done

# Find all subdirectories (assuming each is a Rust project)
find . -maxdepth 1 -type d | while read -r dir; do
    # Skip the root directory
    if [ "$dir" = "." ]; then
        continue
    fi

    echo "Running cargo dylint in $dir"

    # Change to the project directory
    pushd "$dir" >/dev/null

    # Execute cargo dylint with the specified options
    cargo dylint --all --workspace --fix -- --allow-dirty --allow-no-vcs --broken-code --lib

    # Return to the previous directory
    popd >/dev/null
done

# Initialize success counter
success_count=0

# Find all subdirectories containing a Cargo.toml file
find . -maxdepth 2 -type f -name "Cargo.toml" | while read -r cargo_file; do
    dir=$(dirname "$cargo_file")
    echo "Running cargo check in $dir"

    # Change to the directory
    pushd "$dir" >/dev/null

    # Execute cargo check and increment success_count if successful
    if cargo check; then
        success_count=$((success_count + 1))
    fi

    # Return to the previous directory
    popd >/dev/null
done

echo "Total successful cargo check runs: $success_count"
