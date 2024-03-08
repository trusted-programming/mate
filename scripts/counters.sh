#!/bin/bash

suffix=$1
# Find and replace the string in Cargo.toml
sed -i.bak 's|https://github.com/trusted-programming/mate|https://github.com/trusted-programming/count_loop|' Cargo.toml

echo
echo "### FILE OUTPUT ###"
echo

dylint_output=$(cargo dylint --workspace --all -- --exclude tests --exclude benches 2>&1)

echo "$dylint_output"
echo
echo "### FILE OUTPUT END ###"


for_loop_count=$(echo "$dylint_output" | grep -c "warning: found for loop, code: 213423")
iter_count=$(echo "$dylint_output" | grep -c "warning: found iterator, code: 213932")
par_iter_count=$(echo "$dylint_output" | grep -c "warning: found par iterator, code: 213312")

echo
echo "### FINAL RESULTS ###"
echo
echo "for loop occurrences: $for_loop_count"
echo "Total iter occurrences: $iter_count"
echo "Total par iter occurrences: $par_iter_count"
echo
echo "### ALL DONE ###"

# Echo the variables with the suffix to set them in the GitHub environment
echo "for_loop_count_${suffix}=$for_loop_count" >>$GITHUB_ENV
echo "iter_count_${suffix}=$iter_count" >>$GITHUB_ENV
echo "par_iter_count_${suffix}=$par_iter_count" >>$GITHUB_ENV

# Change the string back in Cargo.toml
mv Cargo.toml.bak Cargo.toml
