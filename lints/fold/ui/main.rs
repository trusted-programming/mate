// run-rustfix
fn main() {
    warn_fold_simple();
    get_upload_file_total_size();
}

fn warn_fold_simple() {
    let mut sum = 0;
    let numbers = vec![1, 2, 3, 4, 5];

    numbers.iter().for_each(|&num| {
        sum += num;
    });

    println!("Sum: {}", sum);
}

fn get_upload_file_total_size() -> u64 {
    let some_num = vec![0; 10];
    let mut file_total_size = 0;
    (0..some_num.len()).into_iter().for_each(|_| {
        let (_, upload_size) = (true, 99);
        file_total_size += upload_size;
    });
    file_total_size
}
