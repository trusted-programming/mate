// run-rustfix
use std::thread;
use std::time::Duration;

fn main() {
    (0..100).for_each(|_| thread::sleep(Duration::from_secs(1)));
}
