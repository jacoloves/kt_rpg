use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
    thread,
    time::Duration,
};

use colored::Colorize;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Stats {
    min_attack: u32,
    max_attack: u32,
    min_recovery: u32,
    max_recovery: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Character {
    name: String,
    lv: u32,
    hp: u32,
    max_hp: u32,
    stats: Stats,
    exp: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Monster {
    name: String,
    hp: u32,
    max_hp: u32,
    min_attack: u32,
    max_attack: u32,
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

fn battle(character: &mut Character, monster: &Monster) -> bool {
    let mut rng = rand::thread_rng();
    let mut monster_hp = monster.hp;

    println!("🦕{}が現れた！", monster.name);

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

        thread::sleep(Duration::from_secs(1));

        let attack = rng.gen_range(character.stats.min_attack..=character.stats.max_attack);
        println!("{}の攻撃！ {}のダメージ", character.name, attack);
        monster_hp = monster_hp.saturating_sub(attack);

        println!("monster hp: {}", monster_hp);

        thread::sleep(Duration::from_secs(3));

        if monster_hp == 0 {
            println!("{}", format!("{}を倒した！", monster.name).yellow());
            println!("{}", format!("{}の経験値を得た！💪", monster.exp).blue());

            character.exp += monster.exp;
            check_level_up(character);

            character.hp = character.max_hp;
            save_character(character).expect("セーブ中にエラーが発生しました。");

            return true;
        }

        let attack = rng.gen_range(monster.min_attack..=monster.max_attack);
        println!("{}の攻撃！ {}のダメージ", monster.name, attack);
        character.hp = character.hp.saturating_sub(attack);

        thread::sleep(Duration::from_secs(3));

        if character.hp == 0 {
            println!("{}", format!("{}は倒れた...🚑", character.name).red());
            return false;
        }

        thread::sleep(Duration::from_secs(3));
    }

    false // false for default
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
        println!("😊レベルアップ！ {}レベルになりました！", character.lv);

        let mut rng = rand::thread_rng();
        let hp_increase = rng.gen_range(5..=10);
        character.max_hp += hp_increase;
        character.hp = character.max_hp;
        println!("🙌HPが{}増加しました！", hp_increase);

        let attack_increase = rng.gen_range(1..=3);
        character.stats.min_attack += attack_increase;
        character.stats.max_attack += attack_increase;
        println!("⚔️攻撃力が{}増加しました！", attack_increase);

        let recovery_increase = rng.gen_range(1..=3);

        character.stats.min_recovery += recovery_increase;
        character.stats.max_recovery += recovery_increase;
        println!("🛡️回復力が{}増加しました！", recovery_increase);
    }
}

fn load_monsters() -> io::Result<Vec<Monster>> {
    let path = Path::new("monsters.yaml");

    println!("モンスターのデータを読み込み中...");

    if path.exists() {
        let mut file = File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let monsters: Vec<Monster> = serde_yaml::from_str(&data).unwrap();
        Ok(monsters)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "モンスターデータが見つかりません。",
        ))
    }
}

fn choose_monsters(monsters: &[Monster]) -> Vec<Monster> {
    let high = vec!["ゴブリン", "オオカミ", "スライム", "コウモリ", "ゾンビ"];
    let mid = vec!["スケルトン", "オーク", "ハーピー"];
    let low = vec!["ミノタウロス", "トロール"];
    let rare = vec!["ドラゴン"];

    let mut encounter_pool: Vec<&str> = vec![];

    for name in &high {
        encounter_pool.extend(std::iter::repeat(name).take(5));
    }
    for name in &mid {
        encounter_pool.extend(std::iter::repeat(name).take(3));
    }
    for name in &low {
        encounter_pool.extend(std::iter::repeat(name).take(2));
    }
    for name in &rare {
        encounter_pool.extend(std::iter::repeat(name).take(1));
    }

    let mut rng = thread_rng();
    let mut selected = vec![];

    for _ in 0..10 {
        let name = encounter_pool.choose(&mut rng).unwrap();
        if let Some(monster) = monsters.iter().find(|m| &m.name == *name) {
            selected.push(monster.clone());
        }
    }

    selected
}

fn main() {
    let mut character = load_or_create_character().expect("キャラクターの読み込みに失敗しました。");

    let monsters = load_monsters().expect("モンスターの読み込みに失敗しました");

    let weighted_monsters: Vec<Monster> = choose_monsters(&monsters);

    for monster in weighted_monsters.iter() {
        let win = battle(&mut character, monster);
        if !win {
            println!("ゲームオーバー⚰️");
            break;
        } else {
            println!("ダンジョンを探索中🧭");
            thread::sleep(Duration::from_secs(10));
        }
    }
}
