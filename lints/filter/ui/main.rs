// run-rustfix
fn main() {
    filter_simple();
    filter_simple_flipped();
    filter_simple_macro();
    filter_simple_flipped_macro();
}

fn filter_simple() {
    let items = vec!["apple", "banana", "cherry"];
    let mut one_string = String::new();
    items.iter().for_each(|&item| {
        if item.starts_with('a') {
            one_string.push_str(item);
        }
    });
}

fn filter_simple_flipped() {
    let numbers = vec![1, 2, 3, 4, 5];
    let mut sum = 0;

    numbers.iter().for_each(|&num| {
        if num % 2 == 0 {
            return;
        }
        sum += num;
    });
}

fn filter_simple_macro() {
    let items = vec!["apple", "banana", "cherry"];

    items.iter().for_each(|&item| {
        if item.starts_with('a') {
            println!("Starts with 'a': {}", item);
        }
    });
}

fn filter_simple_flipped_macro() {
    let numbers = vec![1, 2, 3, 4, 5];

    numbers.iter().for_each(|&num| {
        if num % 2 == 0 {
            return;
        }
        println!("Odd number: {}", num);
    });
}
