// run-rustfix
#![allow(dead_code, unused_imports, unused_variables, deprecated)]

use core::ascii;
use futures::io::{self, AsyncWrite, IoSlice};
use futures::task::{Context, Poll};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, LinkedList};
use std::ops::Range;
use std::pin::Pin;
use std::rc::Rc;

struct LocalQueue {}

struct Person {
    name: String,
    age: u32,
}

struct Pantheon {
    tasks: Vec<String>,
}

#[derive(Debug, PartialEq)]
struct Case {
    uid: u32,
    mode: u32,
    priority: u32,
}

#[derive(Clone)]
struct QosCase {
    uid: u64, // Identifier for the Quality of Service case
}

struct ApplicationState {
    foreground_high_qos_cases: Vec<QosCase>,
    background_high_qos_cases: Vec<QosCase>,
}

struct MyWriter;

#[derive(Hash, Eq, PartialEq, Clone)]
struct Id(String);

struct Cmd {
    args: HashMap<Id, Arg>,
}

impl Cmd {
    fn find(&self, key: &Id) -> Option<&Arg> {
        self.args.get(key)
    }
}

struct Arg {
    requires: Vec<(String, Id)>,
}

fn main() {}

// should parallelize
fn simple() {
    (0..100).into_iter().for_each(|x| println!("{:?}", x));
}

// no
fn simple_no_send() {
    let list: Vec<Rc<i32>> = (0..100).map(Rc::new).collect();
    list.iter().for_each(|y| println!("{:?}", y));
}

// no
fn simple_no_send_in_closure_body() {
    let mut list: Vec<Rc<i32>> = (0..100).map(Rc::new).collect();
    (0..100).into_iter().for_each(|x| list.push(Rc::new(x)));
}

// no
fn simple_no_send_in_closure_body_from_inside() {
    let mut list = Vec::new();
    (0..100).into_iter().for_each(|x| list.push(Rc::new(x)));
}

// should parallelize
fn simple_move_inside_closure() {
    let y = 100;
    (0..100)
        .into_iter()
        .for_each(|x| println!("{:?}{:?}", x, y));
}

// no
fn simple_no_push_to_vec() {
    let thread_num = 10;
    let mut locals = Vec::new();
    (0..thread_num).into_iter().for_each(|_| {
        locals.push(LocalQueue {});
    });
}

// no
fn simple_no_push_to_linked_list() {
    let thread_num = 10;
    let mut locals = LinkedList::new();
    (0..thread_num).into_iter().for_each(|_| {
        locals.push_back(2);
    });
}

// no
fn simple_no_insert_to_usize() {
    let thread_num = 10;
    let mut num = 0;
    (0..thread_num).into_iter().for_each(|i| {
        num += i;
    });
}

// should parallelize
fn simple_into_parallel_ref_iterator() {
    let list: LinkedList<i32> = (0..100).collect();
    list.into_iter().for_each(|x| println!("{:?}", x));
}

// should parallelize
fn complex() {
    let vec = vec![1, 2, 3, 4, 5, 6];
    let a = 10;
    let b = 20;
    let c = "Hello";
    let d = 3.14;
    let e = true;
    let person = Person {
        name: String::from("Alice"),
        age: 30,
    };

    (0..10).into_iter().for_each(|x| {
        let sum = x + a + b;
        let message = if e { c } else { "Goodbye" };
        let product = d * (x as f64);
        let person_info = format!("{} is {} years old.", person.name, person.age);
        println!(
            "Sum: {sum}, Message: {message}, Product: {product}, Person: {person_info}, Vec: {vec:?}",
        );
    });
}

// no
fn complex_no_send() {
    let a = Rc::new(10);
    let b = Rc::new(20);
    let c = Rc::new("Hello");
    let d = Rc::new(3.14);
    let e = Rc::new(true);
    let person = Rc::new(Person {
        name: String::from("Alice"),
        age: 30,
    });

    (0..10).into_iter().for_each(|x| {
        let sum = *a + *b;
        let message = if *e { *c } else { "Goodbye" };
        let product = *d * (x as f64);
        let person_info = format!("{} is {} years old.", person.name, person.age);
        println!("Sum: {sum}, Message: {message}, Product: {product}, Person: {person_info}",);
    });
}

// no
fn complex_type_no_trait() {
    let len = 10;
    let path = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
    let name = &path[1..len];
    for byte in name.iter().cloned().flat_map(ascii::escape_default) {
        println!("{}", byte as char);
    }
}

// no
pub fn iter_returned() -> Range<usize> {
    (0..100).into_iter()
}

// no
pub fn iter_returned_not_top_level() {
    match core::char::decode_utf16([1234 as u16, 1232 as u16].iter().copied()).next() {
        Some(Ok(code)) => code,
        _ => return,
    };
}

// no
impl Pantheon {
    fn do_something(&mut self) {
        let tasks = vec!["go", "come", "listen"];
        tasks.into_iter().for_each(|task| {
            self.process(task.to_string());
        });
    }
    fn process(&mut self, task: String) {
        self.tasks.push(task);
    }
}

#[derive(Debug)]
enum MyEnum {
    A,
    B,
    C,
}

// should parallelize
pub fn complex_long_chain() {
    let data = vec![1, 2, 3, 4, 5];
    let multiplier = 2;
    let threshold = 5;
    let my_string = "Hello".to_string();
    let my_enum: MyEnum = MyEnum::B; // Construct the enum variant

    data.iter()
        .map(|&x| {
            let transformed = (x * multiplier + my_string.len() as i32) / 2;
            transformed
        })
        .filter(|&x| {
            let result = x > threshold && x % my_string.len() as i32 == 0;
            result
        })
        .map(|x| {
            let result = match my_enum {
                MyEnum::A => x + 1,
                MyEnum::B => x + 2,
                MyEnum::C => x + 3,
            };
            result
        })
        .filter_map(|x| if x % 3 == 0 { Some(x) } else { None })
        .for_each(|x| {
            println!("{}", x);
        });
}

// should parallelize
fn enumerate_par_iter() {
    let numbers = vec![1, 2, 3, 4, 5];

    numbers.iter().enumerate().for_each(|t| {
        dbg!(t);
    });
}

// no
fn non_allowed_method() {
    let numbers = vec![1, 2, 3];
    numbers.iter().cycle().enumerate().take(10).for_each(|t| {
        dbg!(t);
    });
}

// no
fn simple_fold() {
    let sum;
    let numbers = vec![1, 2, 3, 4, 5];
    sum = numbers.iter().map(|&num| num).fold(0, |mut sum, v| {
        sum += v;
        sum
    });
    println!("Sum: {}", sum);
}

// no
fn request_request_filter() {
    let case = Case {
        uid: 1,
        mode: 10,
        priority: 20,
    };

    let high_qos_cases = vec![
        Case {
            uid: 2,
            mode: 15,
            priority: 25,
        },
        Case {
            uid: 1,
            mode: 5,
            priority: 30,
        },
        Case {
            uid: 3,
            mode: 20,
            priority: 10,
        },
    ];

    let mut down_grade_case = &case;
    let mut swap_case_index_opt: Option<usize> = None;
    (high_qos_cases.iter().enumerate())
        .filter(|(i, swap_case)| {
            down_grade_case.uid == swap_case.uid
                && (down_grade_case.mode < swap_case.mode
                    || down_grade_case.priority < swap_case.priority)
        })
        .for_each(|(i, swap_case)| {
            down_grade_case = swap_case;
            swap_case_index_opt = Some(i)
        });

    println!("Downgrade case: {:?}", down_grade_case);
    println!("Swap case index: {:?}", swap_case_index_opt);
}

// no
impl ApplicationState {
    fn transition_to_background(&mut self, target_uid: u64) {
        let change_state_cases = self
            .background_high_qos_cases
            .iter()
            .cloned()
            .filter(|case| case.uid == target_uid);
        self.foreground_high_qos_cases.extend(change_state_cases);
    }
}

// should parallelize
fn collect_at_end() {
    let people = vec![
        Person {
            name: "Alice".to_string(),
            age: 25,
        },
        Person {
            name: "Bob".to_string(),
            age: 35,
        },
        Person {
            name: "Carol".to_string(),
            age: 32,
        },
    ];

    let names: Vec<String> = people.iter().map(|p| p.name.clone()).collect();

    println!("{:?}", names);
}

struct Tsize {
    send: usize,
}

impl Tsize {
    fn to_no_send(&self) -> TsizeNoSend {
        TsizeNoSend {
            no_send: Rc::new(self.send),
        }
    }
}

#[derive(Debug)]
struct TsizeNoSend {
    no_send: Rc<usize>,
}

// no
fn collect_at_end_no_par() {
    let t_size_vec: Vec<Tsize> = vec![Tsize { send: 32 }, Tsize { send: 42 }];
    let t_size_vec_no_send: Vec<TsizeNoSend> = t_size_vec.iter().map(|t| t.to_no_send()).collect();

    println!("{:?}", t_size_vec_no_send);
}

// should parallelize
impl AsyncWrite for MyWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // Dummy implementation: Pretend we've written the whole buffer
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Dummy implementation: Always say it's flushed
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Dummy implementation: Always say it's closed
        Poll::Ready(Ok(()))
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let buf = bufs
            .iter()
            .find(|b| !b.is_empty())
            .map_or(&[][..], |b| &**b);
        self.poll_write(cx, buf)
    }
}

//should parallelize
fn nested_pars() {
    let used_filtered: HashSet<Id> = HashSet::new();
    let conflicting_keys: HashSet<Id> = HashSet::new();
    let cmd = Cmd {
        args: HashMap::new(),
    };

    let required: Vec<Id> = used_filtered
        .iter()
        .filter_map(|key| cmd.find(key))
        .flat_map(|arg| arg.requires.iter().map(|item| &item.1))
        .filter(|key| !used_filtered.contains(key) && !conflicting_keys.contains(key))
        .chain(used_filtered.iter())
        .cloned()
        .collect();
}

// 1st should parallelize, 2nd no
fn multiple_iter_one_chain() {
    let people = vec![
        Person {
            name: "Alice".to_string(),
            age: 25,
        },
        Person {
            name: "Bob".to_string(),
            age: 35,
        },
        Person {
            name: "Carol".to_string(),
            age: 32,
        },
    ];

    let mut counter = 0;

    let names_over_30: Vec<String> = people
        .iter()
        .filter(|p| p.age > 30)
        .map(|p| p.name.clone())
        .collect::<Vec<String>>()
        .into_iter()
        .map(|name| {
            counter += 1;
            format!("{}: {}", counter, name)
        })
        .collect();

    println!("{:?}", names_over_30);
}

// should parallelize
fn simple_iter_mut() {
    let mut numbers = vec![1, 2, 3, 4, 5];
    numbers.iter_mut().for_each(|num| *num *= 2); // Double each number
    println!("{:?}", numbers);
}

// should parallelize
fn mut_var_declared_in_closure() {
    let numbers = vec![1, 2, 3, 4, 5];
    let doubled_numbers: Vec<i32> = numbers
        .into_iter()
        .map(|num| {
            let mut doubled = num * 2; // Mutable variable inside the closure
            doubled += 1; // Modify the mutable variable
            doubled // Return the modified value
        })
        .collect();
    println!("{:?}", doubled_numbers);
}

// should parallelize
fn return_loop() -> Option<()> {
    let num_workers = 10;
    let locals = vec![1, 2, 3, 4, 5];
    (0..num_workers).into_iter().try_for_each(|index| {
        let item = locals.get(index)?;
        return Some(());
    })?;
    Some(())
}
