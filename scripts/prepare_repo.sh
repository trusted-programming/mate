#!/bin/bash

if ! command -v cargo-dylint &> /dev/null || ! command -v dylint-link &> /dev/null; then
  cargo +stable install cargo-dylint@3.0.0 dylint-link@3.0.0
fi

CARGO_TOML="Cargo.toml"
STRING_TO_APPEND="\n[workspace.metadata.dylint]\nlibraries = [\n    { git = \"https://github.com/trusted-programming/mate\"},\n]"

# Check if Cargo.toml exists
if [ -f "$CARGO_TOML" ]; then
    # Check if the string is already in the file
    if ! grep -qF -- "$STRING_TO_APPEND" "$CARGO_TOML"; then
        # Append the string to Cargo.toml
        echo -e "$STRING_TO_APPEND" >> "$CARGO_TOML"
    else
        echo "String already present in $CARGO_TOML"
    fi
else
    echo "$CARGO_TOML does not exist"
fi

find . -type f -name "Cargo.toml" | while read -r file; do
    # Check if there is a [package] section in the file
    if grep -qE "^\[package\]" "$file"; then
        # Check if rayon is already in the dependencies
        if ! grep -qE "^rayon =" "$file"; then
            # Check if [dependencies] section exists
            if ! grep -qE "^\[dependencies\]" "$file"; then
                # Add [dependencies] section at the end of the file
                echo -e "\n[dependencies]" >> "$file"
            fi
            # Add rayon under the [dependencies] line
            sed -i '/\[dependencies\]/a rayon = "1.9.0"' "$file"
        fi
    fi
done

git config user.name 'test'
git config user.email 'test@test.com'
git add .
git commit -am "test"
