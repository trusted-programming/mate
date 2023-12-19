// run-rustfix
#![allow(unknown_lints)]
#![allow(warn_par_fold_simple)]

#[allow(unused_imports)]
use rayon::prelude::*;

fn main() {
    warn_fold_simple();
}

fn warn_fold_simple() {
    let mut sum = 0;
    let numbers = vec![1, 2, 3, 4, 5];

    numbers.iter().for_each(|&num| {
        sum += num;
    });

    println!("Sum: {}", sum);
}
