// run-rustfix
use std::thread;
use std::time::Duration;

fn main() {}
fn test_for_each() {
    for i in 1..=100 {
        thread::sleep(Duration::from_secs(1));
    }
}
