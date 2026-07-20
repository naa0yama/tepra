# 0009. tepra-web config file discovery

- Status: Accepted
- Date: 2026-07-20
- Deciders: project owner

## Context

`tepra serve` は 3 個の CLI options (`--template-dir` / `--bind` /
`--creator-base`) を持ち、開発時 / 運用時ともに毎回渡す運用負荷が高い。
config file 化により恒久設定を可能にするが、config file の
**探索経路** は複数の設計案があり、ここで決定する必要がある。

前提と制約:

- ADR 0005 で subcommand split (`serve` / `tray` / `install-service`) を決定済。
  primary deployment target は Windows service。
- Windows service 起動時の CWD は sc.exe 登録の `binPath` /
  `workingDirectory` に依存。 tray 経由起動と CWD が異なりうる。
- XDG Base Directory Specification (`~/.config/<app>/`) は Linux / macOS
  の慣例、Windows では `%APPDATA%\<app>\` が慣例。 dual-path 対応は
  探索順序管理と cross-platform 定数の維持を伴う。
- workspace deps に `figment = { features = ["env", "toml"] }` が
  既に宣言済。 未使用の状態。
- ユーザ要求: `tepra serve` 単独 (arg なし) で起動できること。

## Decision

Config file 探索は以下の 2 経路のみに限定する:

1. `--config <PATH>` CLI option で明示指定
   - 絶対 / 相対いずれも可
   - not-found は **error** (silent fallback しない)
2. `--config` 未指定時は CWD 相対 `./tepra.toml` を自動探索
   - 未発見は **silent fallback** (built-in default 適用、ログ出力なし)

XDG_CONFIG_HOME (`~/.config/tepra/config.toml`) や Windows
`%APPDATA%\tepra\config.toml` の自動探索は行わない。 上位ディレクトリ
への探索 (git-style upward search) も行わない。

## Consequences

Positive:

- 実装がシンプル (探索経路 2 個のみ、cross-platform 分岐なし)。
- Windows service deployment は `--config <absolute-path>` を sc.exe
  binPath に含める運用で CWD 依存を排除できる。
- 開発時は project root の `./tepra.toml` を編集するだけで動作確認可能。
- 将来 XDG / APPDATA を追加するのは後方互換な拡張 (既存の
  `--config` 明示 と CWD auto-discovery は温存可能)。
- `figment` の Provider 合成 (Serialized / Toml / Env) と自然に噛み合う。

Negative:

- 標準的な XDG / APPDATA 配置を期待するユーザに驚きあり
  (`~/.config/tepra/` を作っても読まれない)。
  - Mitigation: `--help` epilog と README に自動探索経路を明示。
- Windows service deployment で `--config <absolute>` を渡し忘れると
  default 起動になる (config が無視される事故)。
  - Mitigation: 将来的な `install-service` subcommand 実装で
    `--config` を必須引数として組み込む (別 spec)。

## Alternatives Considered

- **XDG / APPDATA フォールバック追加**
  ( `./tepra.toml` → `$XDG_CONFIG_HOME/tepra/config.toml` →
  `%APPDATA%\tepra\config.toml` の順で探索) —
  cross-platform 分岐と探索順序の維持が必要。 現時点で primary
  user は project 直下で開発、production は sc.exe 経由の絶対 path
  指定で運用する想定のため auto-discovery は不要と判断。 将来的な
  拡張余地は温存する。
- **`--config <PATH>` 明示指定のみ ( auto-discovery なし )** —
  ユーザ要求「毎回引数を渡すのがしんどい」と矛盾する。 開発時の
  `cargo run -p tepra-web -- serve` 単独起動を成立させるには
  auto-discovery が必須。
- **上位ディレクトリ探索** ( git-style upward search from CWD ) —
  予想外の file が読まれる事故リスクがある。 project 構造次第で
  挙動が不定になり、テスト困難。 却下。
- **環境変数 `TEPRA_CONFIG` で config path 指定** — `--config` CLI
  option と役割が重複、 precedence を追加で決める必要が生じ複雑化。
  必要になれば将来追加可能な後方互換な拡張。

## History

- 2026-07-20: initial version
