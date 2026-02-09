# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

kt_rpgは、Rustで実装されたターン制バトルRPGゲームです。キャラクターがモンスターと戦い、経験値を獲得してレベルアップするシンプルなゲームシステムを持っています。

## ビルド・実行コマンド

```bash
# ビルド
cargo build

# 実行
cargo run

# リリースビルド
cargo build --release

# リント
cargo clippy

# フォーマット
cargo fmt
```

## アーキテクチャ

### データ永続化
- **savefile.yaml**: キャラクターデータの保存ファイル（`Character`構造体をシリアライズ）
- **monsters.yaml**: モンスターマスターデータ（ゲーム起動時に読み込み）

### ゲームモード
- **Normal**: 重み付けランダムで選ばれた10体のモンスターと連続バトル。敗北でゲームオーバー
- **Boost**: 指定回数（10/100/500/999）の連続バトル。敗北しても継続、最後に戦績表示

### バトルシステム
- 毎ターン50%の確率で攻撃または回復を実行（ランダム選択）
- モンスターの出現率は4段階（high/mid/low/rare）で重み付け

### 主要な構造体
- `Character`: プレイヤーキャラクター（レベル、HP、攻撃力、回復力、経験値を持つ）
- `Monster`: モンスターデータ（HP、攻撃力、獲得経験値を持つ）
- `Stats`: キャラクターの戦闘ステータス（攻撃・回復の最小/最大値）

### レベルアップ
- 必要経験値: `100 * レベル^2`
- HP、攻撃力、回復力がランダムに上昇
