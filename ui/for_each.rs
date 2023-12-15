// run-rustfix
#[allow(unused_imports)]
use rayon::prelude::*;
use std::thread;
use std::time::Duration;

fn main() {
    for _ in 1..=100 {
        thread::sleep(Duration::from_secs(1));
    }
}
