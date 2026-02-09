use std::{
    fs::File,
    io::{self, stdin, Read, Write},
    path::Path,
    thread,
    time::Duration,
};

use colored::Colorize;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};

// ゲームモード
enum Mode {
    Normal, // 通常モード（既存）
    Boost,  // ブーストモード（既存）
    Stage,  // ステージモード（新規）
}

// ステージを表すenum
// 各ステージには解放に必要なレベル、バトル数、テーマがある
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Grassland = 1, // 草原 (Lv1解放) - 弱い魔物が住む平和な草原
    Forest = 2,    // 森 (Lv8解放) - 野生の獣や妖精が住む森
    Cave = 3,      // 洞窟 (Lv15解放) - アンデッドや闇の生物が住む
    Mountain = 4,  // 山 (Lv25解放) - 強力な魔物が生息する山岳地帯
    Castle = 5,    // 城 (Lv40解放) - 魔王の城、最強の敵が待ち受ける
}

impl Stage {
    // 各ステージの解放に必要なレベルを返す
    fn required_level(&self) -> u32 {
        match self {
            Stage::Grassland => 1,
            Stage::Forest => 8,
            Stage::Cave => 15,
            Stage::Mountain => 25,
            Stage::Castle => 40,
        }
    }

    // ステージの日本語名を返す
    fn name(&self) -> &'static str {
        match self {
            Stage::Grassland => "草原",
            Stage::Forest => "森",
            Stage::Cave => "洞窟",
            Stage::Mountain => "山",
            Stage::Castle => "城",
        }
    }

    // ステージのバトル数を返す（ボス戦を除く）
    fn battle_count(&self) -> usize {
        match self {
            Stage::Grassland => 5,
            Stage::Forest => 7,
            Stage::Cave => 9,
            Stage::Mountain => 11,
            Stage::Castle => 13,
        }
    }

    // 全ステージを配列で返す
    fn all() -> [Stage; 5] {
        [
            Stage::Grassland,
            Stage::Forest,
            Stage::Cave,
            Stage::Mountain,
            Stage::Castle,
        ]
    }

    // ステージ番号からStageを取得
    fn from_number(n: u32) -> Option<Stage> {
        match n {
            1 => Some(Stage::Grassland),
            2 => Some(Stage::Forest),
            3 => Some(Stage::Cave),
            4 => Some(Stage::Mountain),
            5 => Some(Stage::Castle),
            _ => None,
        }
    }
}

// デフォルトのステージ値（互換性のため）
fn default_stage() -> u32 {
    1
}

#[derive(Serialize, Deserialize, Debug)]
struct Stats {
    min_attack: u32,
    max_attack: u32,
    min_recovery: u32,
    max_recovery: u32,
}

// キャラクター構造体
// ステージシステム対応のため、stages_clearedとcurrent_stageを追加
#[derive(Serialize, Deserialize, Debug)]
struct Character {
    name: String,
    lv: u32,
    hp: u32,
    max_hp: u32,
    stats: Stats,
    exp: u32,
    // クリア済みステージ番号のリスト（互換性のためデフォルト値を設定）
    #[serde(default)]
    stages_cleared: Vec<u32>,
    // 現在挑戦中のステージ（未選択時はNone）
    #[serde(default)]
    current_stage: Option<u32>,
}

// モンスター構造体
// ステージシステム対応のため、stageとis_bossを追加
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Monster {
    name: String,
    hp: u32,
    max_hp: u32,
    min_attack: u32,
    max_attack: u32,
    exp: u32,
    // 所属ステージ番号 (1-5)、互換性のためデフォルト値を設定
    #[serde(default = "default_stage")]
    stage: u32,
    // ボスモンスターフラグ
    #[serde(default)]
    is_boss: bool,
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
            stages_cleared: Vec::new(),
            current_stage: None,
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

        // 🎲 decide attack or recovery for random
        if rng.gen_bool(0.5) {
            let attack = rng.gen_range(character.stats.min_attack..=character.stats.max_attack);
            println!("⚔️ {}の攻撃！ {}のダメージ", character.name, attack);
            monster_hp = monster_hp.saturating_sub(attack);
        } else {
            let recovery =
                rng.gen_range(character.stats.min_recovery..=character.stats.max_recovery);
            character.hp = (character.hp + recovery).min(character.max_hp);
            println!("❤️ {}は回復した！ {}のHPを回復", character.name, recovery);
        }

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
        println!("👊{}の攻撃！ {}のダメージ", monster.name, attack);
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

// 次のレベルに必要な経験値を計算する（ドラクエII風）
// 計算式: floor(10 * lv^1.5 + 10 * lv)
// 例: Lv1→2: 20, Lv2→3: 48, Lv10→11: 416, Lv40→41: 2930
fn required_exp_to_level_up(current_lv: u32) -> u32 {
    let lv = current_lv as f64;
    (10.0 * lv.powf(1.5) + 10.0 * lv).floor() as u32
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
        if let Some(monster) = monsters.iter().find(|m| m.name == *name) {
            selected.push(monster.clone());
        }
    }

    selected
}

fn select_mode() -> Mode {
    println!("モードを選択してください:");
    println!("1. 通常モード");
    println!("2. ブーストモード");
    println!("3. ステージモード");

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    match input.trim() {
        "2" => Mode::Boost,
        "3" => Mode::Stage,
        _ => Mode::Normal,
    }
}

// ステージが解放されているかをチェックする
fn is_stage_unlocked(stage: Stage, character_lv: u32) -> bool {
    character_lv >= stage.required_level()
}

// ステージ選択メニューを表示し、選択されたステージを返す
fn select_stage(character: &Character) -> Option<Stage> {
    println!("\n🗺️ ステージを選択してください:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for stage in Stage::all() {
        let stage_num = stage as u32;
        let unlocked = is_stage_unlocked(stage, character.lv);
        let cleared = character.stages_cleared.contains(&stage_num);

        if unlocked {
            let status = if cleared { "✅クリア済" } else { "" };
            println!(
                "{}. {} (Lv{}～, バトル{}+ボス) {}",
                stage_num,
                stage.name().green(),
                stage.required_level(),
                stage.battle_count(),
                status
            );
        } else {
            println!(
                "{}. {} {} (Lv{}で解放)",
                stage_num,
                stage.name().bright_black(),
                "🔒".bright_black(),
                stage.required_level()
            );
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("0. 戻る");
    println!(
        "\n現在のレベル: {} | 経験値: {}",
        character.lv, character.exp
    );

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    match input.trim().parse::<u32>() {
        Ok(0) => None,
        Ok(n) if (1..=5).contains(&n) => {
            if let Some(stage) = Stage::from_number(n) {
                if is_stage_unlocked(stage, character.lv) {
                    Some(stage)
                } else {
                    println!("❌ そのステージはまだ解放されていません。");
                    None
                }
            } else {
                None
            }
        }
        _ => {
            println!("❌ 無効な入力です。");
            None
        }
    }
}

// 指定されたステージに出現する通常モンスターを取得
fn get_stage_monsters(monsters: &[Monster], stage: Stage) -> Vec<Monster> {
    let stage_num = stage as u32;
    monsters
        .iter()
        .filter(|m| m.stage == stage_num && !m.is_boss)
        .cloned()
        .collect()
}

// 指定されたステージのボスモンスターを取得
fn get_boss_monster(monsters: &[Monster], stage: Stage) -> Option<Monster> {
    let stage_num = stage as u32;
    monsters
        .iter()
        .find(|m| m.stage == stage_num && m.is_boss)
        .cloned()
}

// ステージ用のバトルリストを生成（ランダムに通常モンスターを選択）
fn choose_stage_monsters(monsters: &[Monster], stage: Stage) -> Vec<Monster> {
    let stage_monsters = get_stage_monsters(monsters, stage);
    let battle_count = stage.battle_count();

    let mut rng = thread_rng();
    let mut selected = Vec::with_capacity(battle_count);

    for _ in 0..battle_count {
        if let Some(monster) = stage_monsters.choose(&mut rng) {
            selected.push(monster.clone());
        }
    }

    selected
}

fn select_boost_rounds() -> usize {
    println!("Boost Battle 回数を選択してください:");
    println!("1. 10回\n2. 100回\n3. 500回\n4. 999回");

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "1" => 10,
        "2" => 100,
        "3" => 500,
        "4" => 999,
        _ => {
            println!("無効な入力です。デフォルトの10回を選択します。");
            10
        }
    }
}

// ステージモードのバトルを実行
fn run_stage_mode(character: &mut Character, monsters: &[Monster], stage: Stage) {
    let stage_num = stage as u32;
    let total_battles = stage.battle_count();

    println!("\n🏰 ステージ{}: {} に挑戦！", stage_num, stage.name());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("バトル数: {} + ボス戦", total_battles);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 現在挑戦中のステージを記録
    character.current_stage = Some(stage_num);
    save_character(character).expect("セーブ中にエラーが発生しました。");

    // 通常モンスターとのバトル
    let stage_monsters = choose_stage_monsters(monsters, stage);

    for (i, monster) in stage_monsters.iter().enumerate() {
        println!(
            "\n📍 {} を探索中... (バトル {}/{})",
            stage.name(),
            i + 1,
            total_battles
        );
        thread::sleep(Duration::from_secs(2));

        let win = battle(character, monster);
        if !win {
            println!("\n💀 ステージ{}で敗北...", stage.name());
            character.hp = character.max_hp;
            character.current_stage = None;
            save_character(character).expect("セーブ中にエラーが発生しました。");
            return;
        }

        println!("🧭 先へ進む...");
        thread::sleep(Duration::from_secs(3));
    }

    // ボス戦
    println!("\n⚠️ ボスエリアに到達！");
    thread::sleep(Duration::from_secs(2));

    if let Some(boss) = get_boss_monster(monsters, stage) {
        println!("\n👹 ボス戦開始！");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let win = battle(character, &boss);

        if win {
            println!("\n🎊 ステージ{}: {} クリア！", stage_num, stage.name());
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            // クリア済みステージに追加（重複チェック）
            if !character.stages_cleared.contains(&stage_num) {
                character.stages_cleared.push(stage_num);
            }
            character.current_stage = None;
            save_character(character).expect("セーブ中にエラーが発生しました。");

            // 全ステージクリアチェック
            if character.stages_cleared.len() == 5 {
                println!("\n🏆 おめでとうございます！全ステージクリア！");
                println!("あなたは真の勇者です！");
            }
        } else {
            println!("\n💀 ボス {} に敗北...", boss.name);
            character.hp = character.max_hp;
            character.current_stage = None;
            save_character(character).expect("セーブ中にエラーが発生しました。");
        }
    } else {
        println!("❌ ボスモンスターが見つかりませんでした。");
        character.current_stage = None;
        save_character(character).expect("セーブ中にエラーが発生しました。");
    }
}

fn main() {
    let mut character = load_or_create_character().expect("キャラクターの読み込みに失敗しました。");
    let monsters = load_monsters().expect("モンスターの読み込みに失敗しました");

    match select_mode() {
        Mode::Normal => {
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
        Mode::Boost => {
            let rounds = select_boost_rounds();
            let mut victories = 0;

            for i in 0..rounds {
                let monster = choose_monsters(&monsters)
                    .first()
                    .expect("モンスターが見つかりませんでした。")
                    .clone();

                println!("\n🔥 Boostバトル {} / {}:", i + 1, rounds);
                let win = battle(&mut character, &monster);

                if win {
                    victories += 1;
                    println!("🎉 勝利！累計勝利数: {}", victories);
                } else {
                    println!("😵‍💫 敗北... でも再挑戦します！");
                }

                character.hp = character.max_hp;
                save_character(&character).expect("セーブ中にエラーが発生しました。");

                thread::sleep(Duration::from_secs(1));
            }

            println!(
                "\n🚩 Boost Battle 終了！総勝利数: {} / {} | 最終レベル: {} | 経験値: {}\n",
                victories, rounds, character.lv, character.exp
            );
        }
        Mode::Stage => {
            // ステージ選択ループ
            loop {
                if let Some(stage) = select_stage(&character) {
                    run_stage_mode(&mut character, &monsters, stage);

                    println!("\n続けますか？ (y/n)");
                    let mut input = String::new();
                    stdin().read_line(&mut input).unwrap();
                    if input.trim().to_lowercase() != "y" {
                        break;
                    }
                } else {
                    println!("ステージモードを終了します。");
                    break;
                }
            }
        }
    }
}
