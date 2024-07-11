use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
    thread,
    time::Duration,
};

use rand::Rng;
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

fn battle(character: &mut Character, monster: &Monster) {
    let mut rng = rand::thread_rng();
    let mut monster_hp = monster.hp;

    println!("{}が現れた！", monster.name);

    while character.hp > 0 && monster_hp > 0 {
        let attack = rng.gen_range(character.min_attack..=character.max_attack);
        println!("{}の攻撃！ {}のダメージ", character.name, attack);
        monster_hp -= attack;

        if monster_hp <= 0 {
            println!("{}を倒した！", monster.name);
            character.exp += monster.exp;
            println!("経験値{}を獲得した！", monster.exp);
            break;
        }

        let attack = rng.gen_range(monster.min_attack..=monster.max_attack);
        println!("{}の攻撃！ {}のダメージ", monster.name, attack);
        character.hp -= attack;

        if character.hp <= 0 {
            println!("{}は倒れた...", character.name);
            break;
        }

        thread::sleep(Duration::from_secs(3));
    }

    if character.hp > 0 {
        save_character(character).expect("セーブ中にエラーが発生しました。");
    }
}

fn save_character(character: &Character) -> io::Result<()> {
    let path = Path::new("savefile.json");
    let data = serde_json::to_string_pretty(character)?;
    let mut file = File::create(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

fn main() {
    let mut character = load_or_create_character().expect("キャラクターの読み込みに失敗しました。");

    let monster = Monster {
        name: String::from("スライム"),
        hp: 30,
        min_attack: 1,
        max_attack: 3,
        exp: 10,
    };

    battle(&mut character, &monster);
}
