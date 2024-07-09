use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

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

fn load_or_create_character() -> io::Result<Character> {
    let path = Path::new("savefile.json");

    if path.exists() {
        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let character: Character = serde_json::from_str(&data)?;
        Ok(character)
    } else {
        println!("新しいキャラクターを作成します。名前を入力してください:");
        let mut name = String::new();
        io::stdin().read_line(&mut name)?;
        let name = name.trim().to_string();

        let character = Character {
            name,
            lv: 1,
            hp: 50,
            min_attack: 2,
            max_attack: 5,
            min_recovery: 1,
            max_recovery: 3,
            exp: 0,
        };

        let data = serde_json::to_string_pretty(&character)?;
        let mut file = File::create(path)?;
        file.write_all(data.as_bytes())?;
        Ok(character)
    }
}

fn main() {
    println!("Hello, world!");
}
