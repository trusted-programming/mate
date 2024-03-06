// run-rustfix
#![allow(dead_code, unused_imports, unused_variables)]

fn main() {}

struct MyBuilder {
    headers: Vec<(String, String)>,
}

impl MyBuilder {
    fn new() -> MyBuilder {
        MyBuilder {
            headers: Vec::new(),
        }
    }

    fn header(mut self, key: &str, value: &str) -> MyBuilder {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }
}

// no
fn build_request_builder() {
    let headers = vec![("Key1", "Value1"), ("Key2", "Value2")];
    let mut request = MyBuilder::new();

    for (key, value) in headers {
        request = request.header(key, value);
    }
}

// for_each
fn just_loop() {
    for x in 1..=100 {
        println!("{x}");
    }
}

// for_each
fn loop_continue() {
    let vec_a = vec![1, 2, 3];

    for a in vec_a {
        if a == 1 {
            continue;
        }
        dbg!(a);
    }
}

// no
fn loop_break() {
    let vec_a = vec![1, 2, 3];

    for a in vec_a {
        if a == 1 {
            break;
        }
        dbg!(a);
    }
}

// for_each internal
fn nested_loop() {
    let vec_a = vec![1, 2, 3];
    let vec_b = vec![1, 2, 3];

    for a in vec_a {
        for b in &vec_b {
            dbg!(a, b);
        }
    }
}

fn get_upload_file_total_size() -> u64 {
    let some_num = vec![0; 10];
    let mut file_total_size = 0;
    for _ in 0..some_num.len() {
        let (_, upload_size) = (true, 99);
        file_total_size += upload_size;
    }
    file_total_size
}

// no
fn return_loop() {
    let num_workers = 10;
    let locals = vec![1, 2, 3, 4, 5];
    for index in 0..num_workers {
        let item = locals.get(index);
        if !item.is_some() {
            return;
        }
    }
}

// TODO: double capture
