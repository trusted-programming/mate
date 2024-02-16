for_loop_count=0

find . -type f -name '*.rs' | while read -r file; do
    echo "Checking $file"
    for_loop_count_in_file=$(grep -oE 'for\s+(\([^)]+\)|\w+)\s+in\s+[^{]+' "$file" | wc -l)
    echo "Count in file: $for_loop_count_in_file"
    ((for_loop_count += for_loop_count_in_file))
done
echo "for loop occurrences: $for_loop_count"

iter_count=0
iter_mut_count=0
into_iter_count=0

# Iterate over all .rs files in the current directory and subdirectories
find . -type f -name '*.rs' | while read -r file; do
    echo checking $file
    # Count occurrences of iterator methods in the current file
    iter_count_in_file=$(grep -o '\.iter()' "$file" | wc -l)
    iter_mut_count_in_file=$(grep -o '\.iter_mut()' "$file" | wc -l)
    into_iter_count_in_file=$(grep -o '\.into_iter()' "$file" | wc -l)

    # Update total counters
    ((iter_count += iter_count_in_file))
    ((iter_mut_count += iter_mut_count_in_file))
    ((into_iter_count += into_iter_count_in_file))
done

# Print results
echo ".iter() occurrences: $iter_count"
echo ".iter_mut() occurrences: $iter_mut_count"
echo ".into_iter() occurrences: $into_iter_count"

par_iter_count=0
into_par_iter_count=0
par_iter_mut_count=0

# Iterate over all .rs files in the current directory and subdirectories
find . -type f -name '*.rs' | while read -r file; do
    echo checking $file
    par_iter_count_in_file=$(grep -o '\.par_iter()' "$file" | wc -l)
    into_par_iter_count_in_file=$(grep -o '\.into_par_iter()' "$file" | wc -l)
    par_iter_mut_count_in_file=$(grep -o '\.par_iter_mut()' "$file" | wc -l)

    ((par_iter_count += par_iter_count_in_file))
    ((into_par_iter_count += into_par_iter_count_in_file))
    ((par_iter_mut_count += par_iter_mut_count_in_file))
done

echo ".par_iter() occurrences: $par_iter_count"
echo ".into_par_iter() occurrences: $into_par_iter_count"
echo ".par_iter_mut() occurrences: $par_iter_mut_count"
