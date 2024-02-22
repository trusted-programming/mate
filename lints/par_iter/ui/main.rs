// run-rustfix

#[allow(unused_imports)]
use rayon::prelude::*;
use std::collections::LinkedList;
use std::rc::Rc;

fn main() {
    warn_par_iter_simple();
    // warn_par_iter_simple_no_send();
    warn_par_iter_simple_no_send_in_closure_body();
    move_inside_closure();
    warn_par_iter_simple_into_parallel_ref_iterator();
    warn_par_iter_mut_ref();
    warn_par_complex();
    warn_par_complex_no_send()
}

fn warn_par_iter_simple() {
    (0..100).into_iter().for_each(|x| println!("{:?}", x));
}

fn move_inside_closure() {
    let y = 100;
    (0..100)
        .into_iter()
        .for_each(|x| println!("{:?}{:?}", x, y));
}
// fn warn_par_iter_simple_no_send() {
//     let list: Vec<Rc<i32>> = (0..100).map(Rc::new).collect();
//     list.iter().for_each(|y| println!("{:?}", y));
// }

fn warn_par_iter_simple_no_send_in_closure_body() {
    let mut list: Vec<Rc<i32>> = (0..100).map(Rc::new).collect();
    (0..100).into_iter().for_each(|x| list.push(Rc::new(x)));
}

fn warn_par_iter_simple_into_parallel_ref_iterator() {
    let list: LinkedList<i32> = (0..100).collect();
    list.into_iter().for_each(|x| println!("{:?}", x));
}

struct Person {
    name: String,
    age: u32,
}

fn warn_par_complex() {
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

fn warn_par_complex_no_send() {
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

struct LocalQueue {}

fn warn_par_iter_mut_ref() {
    let thread_num = 10;
    let mut locals = Vec::new();
    (0..thread_num).into_iter().for_each(|_| {
        locals.push(LocalQueue {});
    });
}
