use bycat_value::{Date, DateTime, Value, value};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Person {
    name: String,
    age: u8,
}

fn main() {
    let output = bycat_value::map! {
        "name": "Rasmus"
    };
    // let value: Value = value!("Hello, World!");

    // let date: DateTime = "2015-12-20 23:17:30+02:00".parse().unwrap();

    // println!("Value: {:?}", date);

    // let date: chrono::DateTime<chrono::FixedOffset> = date.try_into().unwrap();

    // println!("Value: {:?}", date);

    // let user = bycat_value::to_value(Person {
    //     name: "Rasmus".to_string(),
    //     age: 41,
    // })
    // .unwrap();

    // let user = value!({
    //     "name": "Rasmus",
    //     "age": 42
    // });

    // println!("Name {}", user["name"]);

    // let user: Person = bycat_value::from_value(user).unwrap();

    // println!("{:?}", user);
}
