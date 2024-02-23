// run-rustfix

use core::ascii;
#[allow(unused_imports)]
use rayon::prelude::*;
// use std::collections::LinkedList;
use std::rc::Rc;
struct LocalQueue {}

struct Person {
    name: String,
    age: u32,
}

fn main() {
    simple();
    simple_no_send();
    simple_no_send_in_closure_body();
    simple_move_inside_closure();
    simple_mut_ref();
    // simple_into_parallel_ref_iterator();
    complex();
    complex_no_send();
    complex_type_no_trait();
}

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

// should parallelize
fn simple_move_inside_closure() {
    let y = 100;
    (0..100)
        .into_iter()
        .for_each(|x| println!("{:?}{:?}", x, y));
}

// no
fn simple_mut_ref() {
    let thread_num = 10;
    let mut locals = Vec::new();
    (0..thread_num).into_iter().for_each(|_| {
        locals.push(LocalQueue {});
    });
}

// should parallelize
// fn simple_into_parallel_ref_iterator() {
//     let list: LinkedList<i32> = (0..100).collect();
//     list.into_iter().for_each(|x| println!("{:?}", x));
// }

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
