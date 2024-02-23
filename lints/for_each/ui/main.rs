// run-rustfix

fn main() {
    just_loop();
    // build_request_builder();
    loop_continue();
    nested_loop();
    loop_break();
    build_request_builder();
}

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
