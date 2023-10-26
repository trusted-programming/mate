#![allow(dead_code)]

fn basic_fold_1() -> i32 {
    let xs = vec![1, 2, 3, 4, 5];
    let mut x = 0;
    for v in xs.into_iter() {
        x += v;
    }
    x
}

fn basic_fold_2() -> i32 {
    let xs = vec![1, 2, 3, 4, 5];
    let mut x = 0;
    xs.into_iter().for_each(|v| {
        x += v;
    });
    x
}

fn basic_fold_3() -> i32 {
    let xs = vec![1, 2, 3, 4, 5];
    let mut x = 0;
    x += xs.into_iter().fold(0, |mut x, v| {
        x += v;
        x
    });
    x
}

// fn basic_fold_4() -> i32 {
//     let xs = vec![1, 2, 3, 4, 5];
//     let mut x = 0;
//     x += xs.into_par_iter().reduce(|| 0, |mut x, v| {
//         x += v;
//         x
//     });
//     x
// }

fn is_valid(_x: usize) -> bool {
    true
}

fn filter_fold_1() -> bool {
    let xs = vec![1, 2, 3, 4, 5];
    let ys = vec![6, 7, 8, 9, 10];

    let mut res = true;
    for (x, y) in xs.iter().zip(ys.iter()) {
        if is_valid(x + y) {
            res &= x + y < 5;
        }
    }
    res
}

fn filter_fold_2() -> bool {
    let xs = vec![1, 2, 3, 4, 5];
    let ys = vec![6, 7, 8, 9, 10];

    let mut res = true;
    xs.iter().zip(ys.iter()).for_each(|(x, y)| {
        if is_valid(x + y) {
            res &= x + y < 5;
        }
    });
    res
}

fn filter_fold_3() -> bool {
    let xs = vec![1, 2, 3, 4, 5];
    let ys = vec![6, 7, 8, 9, 10];

    let mut res = true;
    xs.iter()
        .zip(ys.iter())
        .filter(|&(x, y)| is_valid(x + y))
        .for_each(|(x, y)| {
            res &= x + y < 5;
        });

    res
}

fn filter_fold_4() -> bool {
    let xs = vec![1, 2, 3, 4, 5];
    let ys = vec![6, 7, 8, 9, 10];

    let mut res = true;
    res &= xs.iter()
        .zip(ys.iter())
        .filter(|&(x, y)| is_valid(x + y))
        .map(|(x, y)| x + y < 5)
        .fold(true, |mut res, v| {
            res &= v;
            res
        });

    res
}

// fn filter_fold_5() -> bool {
//     let xs = vec![1, 2, 3, 4, 5];
//     let ys = vec![6, 7, 8, 9, 10];
//
//     let mut res = true;
//     res &= xs.par_iter()
//         .zip(ys.par_iter())
//         .filter(|&(x, y)| is_valid(x + y))
//         .map(|(x, y)| x + y < 5)
//         .reduce(|| true, |mut res, v| {
//             res &= v;
//             res
//         });
//     res
// }

fn main() {}
