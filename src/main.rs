use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
struct Character {
    name: String,
    lv: u32,
    hp: i32,
    min_attack: i32,
    max_attack: i32,
    min_recovery: i32,
    max_recovery: i32,
    exp: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Monster {
    name: String,
    hp: i32,
    min_attack: i32,
    max_attack: i32,
    exp: u32,
}

fn main() {
    println!("Hello, world!");
}
