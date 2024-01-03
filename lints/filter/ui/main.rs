// run-rustfix
fn main() {
    filter_simple_macro();
    filter_simple_flipped_macro();
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
