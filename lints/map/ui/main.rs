// run-rustfix
fn main() {
    warn_vec();
    warn_hashmap();
    warn_hashset();
    warn_btreemap();
    warn_btreeset();
}

fn warn_vec() {
    let mut data = vec![];
    let numbers = vec![1, 2, 3, 4, 5];
    numbers.iter().for_each(|&num| {
        data.push(num * 3);
    });

    println!("Data: {:?}", data);
}

fn warn_hashmap() {
    use std::collections::HashMap;

    let mut data = HashMap::new();
    let numbers = vec![1, 2, 3, 4, 5];
    numbers.iter().for_each(|&num| {
        data.insert(num, num.to_string());
    });

    println!("Data: {:?}", data);
}

fn warn_hashset() {
    use std::collections::HashSet;

    let mut data = HashSet::new();
    let numbers = vec![1, 2, 3, 4, 5];
    numbers.iter().for_each(|&num| {
        data.insert(num);
    });

    println!("Data: {:?}", data);
}

fn warn_btreemap() {
    use std::collections::BTreeMap;

    let mut data = BTreeMap::new();
    let numbers = vec![1, 2, 3, 4, 5];
    numbers.iter().for_each(|&num| {
        data.insert(num, num.to_string());
    });

    println!("Data: {:?}", data);
}

fn warn_btreeset() {
    use std::collections::BTreeSet;

    let mut data = BTreeSet::new();
    let numbers = vec![1, 2, 3, 4, 5];
    numbers.iter().for_each(|&num| {
        data.insert(num);
    });

    println!("Data: {:?}", data);
}
