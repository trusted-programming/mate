// run-rustfix
fn main() {
    warn_fold_simple();
    warn_fold_vec();
    warn_fold_hashmap();
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

fn warn_fold_vec() {
    let mut data = vec![];
    let numbers = vec![1, 2, 3, 4, 5];
    numbers.iter().for_each(|&num| {
        data.push(num * 3);
    });

    println!("Data: {:?}", data);
}

fn warn_fold_hashmap() {
    use std::collections::HashMap;

    let mut data = HashMap::new();
    let numbers = vec![1, 2, 3, 4, 5];
    numbers.iter().for_each(|&num| {
        data.insert(num, num.to_string());
    });

    println!("Data: {:?}", data);
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
