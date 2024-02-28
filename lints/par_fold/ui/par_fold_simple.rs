// run-rustfix

#[allow(unused_imports)]
use rayon::prelude::*;

fn main() {
    warn_fold_simple();
    warn_fold_vec();
    warn_fold_hashmap();
}

fn warn_fold_simple() {
    let mut sum = 0;
    let numbers = vec![1, 2, 3, 4, 5];
    sum += numbers.iter().map(|&num| num).fold(0, |mut sum, v| {
        sum += v;
        sum
    });

    println!("Sum: {}", sum);
}

fn warn_fold_vec() {
    let mut data = vec![];
    let numbers = vec![1, 2, 3, 4, 5];
    data = numbers.iter().fold(data, |mut data, &num| {
        data.push(num * 3);
        data
    });

    println!("Data: {:?}", data);
}

fn warn_fold_hashmap() {
    use std::collections::HashMap;

    let mut data = HashMap::new();
    let numbers = vec![1, 2, 3, 4, 5];
    data = numbers.iter().fold(data, |mut data, &num| {
        data.insert(num, num.to_string());
        data
    });

    println!("Data: {:?}", data);
}
