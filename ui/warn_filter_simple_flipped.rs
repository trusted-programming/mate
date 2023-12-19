// run-rustfix
#[allow(unused_imports)]
use rayon::prelude::*;

fn main() {
    // for_each_simple();
}

// fn for_each_simple() {
//     let numbers = vec![1, 2, 3, 4, 5];

//     numbers.iter().for_each(|&num| {
//         if num % 2 == 0 {
//             return; // Early return for even numbers
//         }
//         // Further processing for odd numbers
//         println!("Odd number: {}", num);
//     });
// }
