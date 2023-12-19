// run-rustfix
#[allow(unused_imports)]
use rayon::prelude::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() {
    just_loop();
    build_request_builder();
    // loop_continue();
    // nested_loop();
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

fn just_loop() {
    for _ in 1..=100 {
        thread::sleep(Duration::from_secs(1));
    }
}

// fn loop_continue() {
//     let vec_a = vec![1, 2, 3];

//     for a in vec_a {
//         if a == 1 {
//             continue;
//         }
//         dbg!(a);
//     }
// }

// fn nested_loop() {
//     let vec_a = vec![1, 2, 3];
//     let vec_b = vec![1, 2, 3];

//     for a in vec_a {
//         for b in &vec_b {
//             dbg!(a, b);
//         }
//     }
// }
