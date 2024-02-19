#!/bin/bash

suffix=$1

#!/bin/bash
rs_files=($(find -L . -type d -name target -prune -o -type f -name '*.rs' -print))

for_loop_count=0
echo "### COUNTING FOR LOOPS ###"
for file in "${rs_files[@]}"; do
    echo "Checking: $file"
    for_loop_count_in_file=$(grep -oE 'for\s+(\([^)]+\)|\w+)\s+in\s+[^{]+' "$file" | wc -l)
    echo "Count in file: $for_loop_count_in_file"
    ((for_loop_count += for_loop_count_in_file))
done

iter_count=0
iter_mut_count=0
into_iter_count=0

echo "### COUNTING ITERS ###"
for file in "${rs_files[@]}"; do
    echo "Checking: $file"
    iter_count_in_file=$(grep -oE '\.iter\(\)' "$file" | wc -l)
    iter_mut_count_in_file=$(grep -oE '\.iter_mut\(\)' "$file" | wc -l)
    into_iter_count_in_file=$(grep -oE '\.into_iter\(\)' "$file" | wc -l)
    echo "iter_count_in_file: $iter_count_in_file"
    echo "iter_mut_count_in_file: $iter_mut_count_in_file"
    echo "into_iter_count_in_file: $into_iter_count_in_file"

    ((iter_count += iter_count_in_file))
    ((iter_mut_count += iter_mut_count_in_file))
    ((into_iter_count += into_iter_count_in_file))
done
general_iter_count=$((iter_count + iter_mut_count + into_iter_count))

par_iter_count=0
into_par_iter_count=0
par_iter_mut_count=0

echo "### COUNTING PAR ITERS ###"
for file in "${rs_files[@]}"; do
    echo "Checking: $file"
    par_iter_count_in_file=$(grep -oE '\.par_iter\(\)' "$file" | wc -l)
    into_par_iter_count_in_file=$(grep -oE '\.into_par_iter\(\)' "$file" | wc -l)
    par_iter_mut_count_in_file=$(grep -oE '\.par_iter_mut\(\)' "$file" | wc -l)
    echo "par_iter_count_in_file: $par_iter_count_in_file"
    echo "into_par_iter_count_in_file: $into_par_iter_count_in_file"
    echo "par_iter_mut_count_in_file: $par_iter_mut_count_in_file"
    ((par_iter_count += par_iter_count_in_file))
    ((into_par_iter_count += into_par_iter_count_in_file))
    ((par_iter_mut_count += par_iter_mut_count_in_file))
done
par_iter_total_count=$((par_iter_count + into_par_iter_count + par_iter_mut_count))

echo
echo "### FINAL RESULTS ###"
echo
echo "for loop occurrences: $for_loop_count"
echo ".iter() occurrences: $iter_count"
echo ".iter_mut() occurrences: $iter_mut_count"
echo ".into_iter() occurrences: $into_iter_count"
echo "Total .iter*() occurrences: $general_iter_count"
echo ".par_iter() occurrences: $par_iter_count"
echo ".into_par_iter() occurrences: $into_par_iter_count"
echo ".par_iter_mut() occurrences: $par_iter_mut_count"
echo "Total .par_iter*() occurrences: $par_iter_total_count"
echo
echo "### ALL DONE ###"

# Echo the variables with the suffix to set them in the GitHub environment
echo "for_loop_count_${suffix}=$for_loop_count" >>$GITHUB_ENV
echo "iter_count_${suffix}=$iter_count" >>$GITHUB_ENV
echo "iter_mut_count_${suffix}=$iter_mut_count" >>$GITHUB_ENV
echo "into_iter_count_${suffix}=$into_iter_count" >>$GITHUB_ENV
echo "general_iter_count_${suffix}=$((iter_count + iter_mut_count + into_iter_count))" >>$GITHUB_ENV
echo "par_iter_count_${suffix}=$par_iter_count" >>$GITHUB_ENV
echo "into_par_iter_count_${suffix}=$into_par_iter_count" >>$GITHUB_ENV
echo "par_iter_mut_count_${suffix}=$par_iter_mut_count" >>$GITHUB_ENV
echo "par_iter_total_count_${suffix}=$((par_iter_count + into_par_iter_count + par_iter_mut_count))" >>$GITHUB_ENV
