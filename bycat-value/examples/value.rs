use bycat_value::{Date, DateTime, Value};

fn main() {
    let value: Value = "Hello, World!".into();

    let date: DateTime = "2015-12-20 10:22:30z".parse().unwrap();

    println!("Value: {}", date);
}
