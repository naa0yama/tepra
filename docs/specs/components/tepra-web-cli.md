# tepra-web CLI

`crates/tepra-web/src/cli.rs` が定義する `tepra` バイナリの CLI 仕様。
clap derive で subcommand 分割し、 Linux / Windows の配備差分を
single binary 内に閉じ込める。

## 構造

```
tepra <SUBCOMMAND>
  serve       全プラットフォーム / HTTP サーバ起動
  version     全プラットフォーム / ビルドメタ表示
  tray        Windows のみ ( ADR 0005 ) / トレイ常駐 + serve 内蔵
  install-service / uninstall-service  Windows のみ
```

OS gate は `#[cfg(windows)]` で表現し、 Linux ビルドでは
`tray-icon` / `windows-service` 等の Windows 専用 crate を pull しない。

## `serve` arguments

```
tepra serve
  [--config <PATH>]
  [--template-dir <PATH>]
  [--bind <ADDR>]
  [--creator-base <URL>]
```

全 option が `Option<T>` 型。clap 側は default 値を持たず、後段の
config cascade (下記参照) で最終値を決定する。

- `--config <PATH>` — config file の明示指定。指定時にファイルが存在しない
  場合はエラー終了 (silent fallback なし)
- `--template-dir <PATH>` ( Option ) — ラベルテンプレートファイル格納
  ディレクトリ。省略時は cascade で `templates/` が適用される
- `--bind <ADDR>` — HTTP listen address
- `--creator-base <URL>` — Creator WebAPI の base URL

## Configuration cascade

優先度は上ほど強い:

1. CLI arg ( `--template-dir` 等で明示された値 )
2. Env var ( `TEPRA_*` prefix )
3. Config file ( `tepra.toml` )
4. Built-in default

### Built-in default 値

| フィールド     | デフォルト値             |
| -------------- | ------------------------ |
| `template_dir` | `templates/` (CWD 相対)  |
| `bind`         | `0.0.0.0:3000`           |
| `creator_base` | `http://localhost:29108` |

### TOML schema

```toml
# tepra serve config file
# CLI arg > env var (TEPRA_*) > file > built-in default の順で上書き。

template_dir = "templates"
bind = "0.0.0.0:3000"
creator_base = "http://localhost:29108"
```

- field 名は TOML 慣例で `snake_case`
- 全 field optional (省略時は built-in default)
- コメント記述可能 ( `#` )

### Env var 一覧

| 変数名               | 対応フィールド |
| -------------------- | -------------- |
| `TEPRA_TEMPLATE_DIR` | `template_dir` |
| `TEPRA_BIND`         | `bind`         |
| `TEPRA_CREATOR_BASE` | `creator_base` |

`figment::providers::Env::prefixed("TEPRA_")` の慣例に従う。

### Config file 探索

2 経路のみ ( ADR 0009 参照 ):

1. `--config <PATH>` 明示 — 絶対 / 相対いずれも可。ファイルが存在しない
   場合は **error** (silent fallback なし)
2. `--config` 未指定 — CWD 相対 `./tepra.toml` を自動探索。ファイルが
   存在しない場合は **silent fallback** (built-in default を適用、ログ出力なし)

## `version`

ビルド時の `CARGO_PKG_VERSION` を 1 行で stdout 出力して exit。
`tepra_web::app_version()` を再利用。

## エントリポイント

`crates/tepra-web/src/main.rs`:

1. `Cli::parse()` で引数 parse
2. `Commands::Serve(args)` の場合:
   - `load_config(&args)` で `ServeConfig` を合成 (figment cascade)
   - `init_telemetry()` 呼び出し後、 effective config を `INFO` ログ 1 行 emit
   - `ReqwestTepraClient::new(config.creator_base)` を `Arc` で生成
   - `AppState::new_with_template_dir(client, config.template_dir)` を構築
   - 4 つの router builder を `.merge()` し、 `.layer(TraceLayer::new_for_http())`
     を付加して `config.bind` に bind し、 `axum::serve` で起動
3. `Commands::Version` の場合: バージョン文字列を 1 行出力

## 関連 ADR

- `docs/adr/latest/0005-cli-subcommand-split.md` — subcommand 分割の判断
- `docs/adr/latest/0006-http-observability-with-tower-http-tracelayer.md` — TraceLayer 導入の判断
- `docs/adr/latest/0009-tepra-web-config-file-discovery.md` — config file 探索仕様の判断
