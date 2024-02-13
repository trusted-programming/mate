// run-rustfix

#[allow(unused_imports)]
use rayon::prelude::*;
use std::collections::LinkedList;

fn main() {
    warn_par_iter_simple();
    warn_par_iter_simple_no_send();
    // warn_par_iter();
}

fn warn_par_iter_simple() {
    (0..100).into_iter().for_each(|x| println!("{:?}", x));
}

fn warn_par_iter_simple_no_send() {
    let list: LinkedList<i32> = (0..100).collect();
    list.into_iter().for_each(|x| println!("{:?}", x));
}

// struct LocalQueue {}

// fn warn_par_iter() {
//     let thread_num = 10;
//     let mut locals = Vec::new();
//     (0..thread_num).into_iter().for_each(|_| {
//         locals.push(LocalQueue {});
//     });
// }
