// run-rustfix
#![allow(dead_code)]
#![allow(unused_imports)]

use core::ascii;
use rayon::prelude::*;
use std::collections::LinkedList;
use std::ops::Range;
use std::rc::Rc;

struct LocalQueue {}

struct Person {
    name: String,
    age: u32,
}

struct Pantheon {
    tasks: Vec<String>,
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

// // no
// pub fn complex_long_chain_no_par() {
//     let words = vec!["apple", "banana", "cherry", "date"];
//     let numbers = vec![1, 2, 3, 4];
//     let suffixes = vec!["st", "nd", "rd", "th"];

//     words
//         .into_iter()
//         .enumerate()
//         .map(|(i, word)| {
//             let number = *numbers.get(i).unwrap_or(&0) * 2;
//             let suffix = suffixes.get(i).unwrap_or(&"th");
//             (word, number, suffix)
//         })
//         .filter(|(word, number, _)| !word.contains('a') || *number > 5)
//         .map(|(word, number, suffix)| format!("{}-{}{}", word, number, suffix))
//         .enumerate()
//         .map(|(i, s)| if i % 2 == 0 { s.to_uppercase() } else { s })
//         .take(2)
//         .for_each(|x| {
//             println!("{x}");
//         });
// }

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
    let my_enum = MyEnum::B; // Construct the enum variant

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
// TODO: add test with invalid method in it eg. peekable
// TODO: closure with non send sync argument
// TODO: multiple iter in one chain
