// run-rustfix
#![allow(dead_code, unused_variables, deprecated)]

use rayon::prelude::*;

fn main() {}

fn find_simple() {
    let bufs = vec![vec![1], vec![2], vec![2]];
    let buf = bufs
        .par_iter()
        .find(|b| !b.is_empty())
        .map_or(&[][..], |b| &**b);
}

fn position_simple() {
    let bufs = vec![vec![1], vec![2], vec![2]];
    let buf = bufs.par_iter().position(|b| !b.is_empty());
}
