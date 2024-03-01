// run-rustfix
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use core::ascii;
use rayon::prelude::*;
use std::collections::LinkedList;
use std::ops::Range;
use std::rc::Rc;

fn main() {}

// // should parallelize
// fn collect_at_end() {
//     let people = vec![
//         Person {
//             name: "Alice".to_string(),
//             age: 25,
//         },
//         Person {
//             name: "Bob".to_string(),
//             age: 35,
//         },
//         Person {
//             name: "Carol".to_string(),
//             age: 32,
//         },
//     ];

//     let names: Vec<String> = people.iter().map(|p| p.name.clone()).collect();

//     println!("{:?}", names);
// }

// // 1st should parallelize, 2nd no
// fn multiple_iter_one_chain() {
//     let people = vec![
//         Person {
//             name: "Alice".to_string(),
//             age: 25,
//         },
//         Person {
//             name: "Bob".to_string(),
//             age: 35,
//         },
//         Person {
//             name: "Carol".to_string(),
//             age: 32,
//         },
//     ];

//     let mut counter = 0;

//     let names_over_30: Vec<String> = people
//         .iter()
//         .filter(|p| p.age > 30)
//         .map(|p| p.name.clone())
//         .collect::<Vec<String>>()
//         .into_iter()
//         .map(|name| {
//             counter += 1;
//             format!("{}: {}", counter, name)
//         })
//         .collect();

//     println!("{:?}", names_over_30);
// }

// // no
// pub fn complex_long_chain_no_par() {
//     let words = vec!["apple", "banana", "cherry", "date"];
//     let numbers = vec![1, 2, 3, 4];
//     let suffixes = vec!["st", "nd", "rd", "th"];

//     words
//         .into_iter()
//         .enumerate()
//         .map(|(i, word)| {
//             let number = *numbers.get(i).unwrap_or(&0) * 2;
//             let suffix = suffixes.get(i).unwrap_or(&"th");
//             (word, number, suffix)
//         })
//         .filter(|(word, number, _)| !word.contains('a') || *number > 5)
//         .map(|(word, number, suffix)| format!("{}-{}{}", word, number, suffix))
//         .enumerate()
//         .map(|(i, s)| if i % 2 == 0 { s.to_uppercase() } else { s })
//         .take(2)
//         .for_each(|x| {
//             println!("{x}");
//         });
// }

// // should parallelize
// fn mut_var_declared_in_closure() {
//     let numbers = vec![1, 2, 3, 4, 5];
//     let doubled_numbers: Vec<i32> = numbers
//         .into_iter()
//         .map(|num| {
//             let mut doubled = num * 2; // Mutable variable inside the closure
//             doubled += 1; // Modify the mutable variable
//             doubled // Return the modified value
//         })
//         .collect();
//     println!("{:?}", doubled_numbers);
// }

// // no
// fn simple_fold() {
//     let sum;
//     let numbers = vec![1, 2, 3, 4, 5];
//     sum = numbers.iter().map(|&num| num).fold(0, |mut sum, v| {
//         sum += v;
//         sum
//     });
//     println!("Sum: {}", sum);
// }

// // should parallelize
// fn simple_iter_mut() {
//     let mut numbers = vec![1, 2, 3, 4, 5];
//     numbers.iter_mut().for_each(|num| *num *= 2); // Double each number
//     println!("{:?}", numbers);
// }

// // should parallelize
// fn no_iter_keywords() {
//     (0..100).for_each(|x| println!("{x}"));
// }

// // should parallelize
// pub fn iter_returned_in_variable() {
//     let _: Range<i32> = (0..100).into_iter();
// }
