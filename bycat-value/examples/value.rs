use bycat_value::{Date, Value};

fn main() {
    let value: Value = "Hello, World!".into();

    let date: Date = "2015-12-20-10".parse().unwrap();

    println!("Value: {}", date);
}
