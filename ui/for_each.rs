// run-rustfix
#[allow(unused_imports)]
use rayon::prelude::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() {
    for _ in 1..=100 {
        thread::sleep(Duration::from_secs(1));

        let _res = build_request_builder();
    }
}

fn build_request_builder() -> (String, String) {
    let mut a_map = HashMap::new();
    a_map.insert("a".to_string(), "b".to_string());
    a_map.insert("c".to_string(), "d".to_string());
    a_map.insert("e".to_string(), "f".to_string());

    let mut request = (String::new(), String::new());
    for (key, value) in a_map.iter() {
        request = (key.clone(), value.clone());
    }
    request
}
