fn main() {
    let data: [i32; 5] = [1, 2, 3, 4, 5];
    let sum: i32 = data.iter().sum();
    println!("The sum is: {}", sum);
}
