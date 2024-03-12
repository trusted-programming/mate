// run-rustfix

#[allow(unused_imports)]
use rayon::prelude::*;

fn main() {
    warn_fold_simple();
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
