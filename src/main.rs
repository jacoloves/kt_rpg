use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
    thread,
    time::Duration,
};

use colored::Colorize;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Stats {
    min_attack: i32,
    max_attack: i32,
    min_recovery: i32,
    max_recovery: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Character {
    name: String,
    lv: u32,
    hp: i32,
    max_hp: i32,
    stats: Stats,
    exp: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Monster {
    name: String,
    hp: i32,
    max_hp: i32,
    min_attack: i32,
    max_attack: i32,
    exp: u32,
}

fn load_or_create_character() -> io::Result<Character> {
    let path = Path::new("savefile.yaml");

    if path.exists() {
        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let character: Character = serde_yaml::from_str(&data).unwrap();
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
            max_hp: 50,
            stats: Stats {
                min_attack: 2,
                max_attack: 5,
                min_recovery: 1,
                max_recovery: 3,
            },
            exp: 0,
        };

        let data = serde_yaml::to_string(&character).unwrap();
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
        println!(
            "{} HP: {}/{} | {} HP: {}/{}",
            character.name.green(),
            character.hp.to_string().green(),
            character.max_hp.to_string().green(),
            monster.name.red(),
            monster_hp.to_string().red(),
            monster.max_hp.to_string().red()
        );

        let attack = rng.gen_range(character.stats.min_attack..=character.stats.max_attack);
        println!("{}の攻撃！ {}のダメージ", character.name, attack);
        monster_hp -= attack;

        if monster_hp <= 0 {
            let victory_message = format!("{}を倒した！", monster.name);
            println!("{}", victory_message.yellow());
            let get_exp_message = format!("{}の経験値を獲得した！", monster.exp);
            character.exp += monster.exp;
            println!("{}", get_exp_message.blue());
            break;
        }

        let attack = rng.gen_range(monster.min_attack..=monster.max_attack);
        println!("{}の攻撃！ {}のダメージ", monster.name, attack);
        character.hp -= attack;

        if character.hp <= 0 {
            let lose_message = format!("{}を倒れた...", character.name);
            println!("{}", lose_message.red());
            break;
        }

        thread::sleep(Duration::from_secs(3));
    }

    if character.hp > 0 {
        character.hp = character.max_hp;
        save_character(character).expect("セーブ中にエラーが発生しました。");
    }
}

fn save_character(character: &Character) -> io::Result<()> {
    let path = Path::new("savefile.yaml");
    let data = serde_yaml::to_string(character).unwrap();
    let mut file = File::create(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

#[allow(dead_code)]
fn required_exp_to_level_up(current_lv: u32) -> u32 {
    100 * current_lv * current_lv
}

#[allow(dead_code)]
fn check_level_up(character: &mut Character) {
    while character.exp >= required_exp_to_level_up(character.lv) {
        character.exp -= required_exp_to_level_up(character.lv);
        character.lv += 1;
        println!("レベルアップ！ {}レベルになりました！", character.lv);

        let mut rng = rand::thread_rng();
        let hp_increase = rng.gen_range(5..=10);
        character.max_hp += hp_increase;
        character.hp = character.max_hp;
        println!("HPが{}増加しました！", hp_increase);

        let attack_increase = rng.gen_range(1..=3);
        character.stats.min_attack += attack_increase;
        character.stats.max_attack += attack_increase;
        println!("攻撃力が{}増加しました！", attack_increase);

        let recovery_increase = rng.gen_range(1..=3);

        character.stats.min_recovery += recovery_increase;
        character.stats.max_recovery += recovery_increase;
        println!("回復力が{}増加しました！", recovery_increase);
    }
}

fn main() {
    let mut character = load_or_create_character().expect("キャラクターの読み込みに失敗しました。");

    let monster = Monster {
        name: String::from("スライム"),
        hp: 30,
        max_hp: 30,
        min_attack: 1,
        max_attack: 3,
        exp: 10,
    };

    battle(&mut character, &monster);
}
