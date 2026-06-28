# tepra-web CLI

`crates/tepra-web/src/cli.rs` が定義する `tepra-api` バイナリの CLI 仕様。
clap derive で subcommand 分割し、 Linux / Windows の配備差分を
single binary 内に閉じ込める。

## 構造

```
tepra-api <SUBCOMMAND>
  serve       全プラットフォーム / HTTP サーバ起動
  version     全プラットフォーム / ビルドメタ表示
  tray        Windows のみ ( ADR 0005 ) / トレイ常駐 + serve 内蔵
  install-service / uninstall-service  Windows のみ
```

OS gate は `#[cfg(windows)]` で表現し、 Linux ビルドでは
`tray-icon` / `windows-service` 等の Windows 専用 crate を pull しない。

## `serve` arguments

```
tepra-api serve \
  --template-dir <PATH> \
  [--bind <ADDR>] \
  [--creator-base <URL>]
```

- `--template-dir <PATH>` ( required ) — ラベルテンプレートファイル格納
  ディレクトリ。 `GET /api/templates` で列挙、 `template/importframe` で
  読み込む
- `--bind <ADDR>` — HTTP listen address ( default `0.0.0.0:3000` )
- `--creator-base <URL>` — Creator `WebAPI` の base URL
  ( default `http://localhost:29108` )

## `version`

ビルド時の `CARGO_PKG_VERSION` を 1 行で stdout 出力して exit。
`tepra_web::app_version()` を再利用。

## エントリポイント

`crates/tepra-web/src/main.rs`:

1. `Cli::parse()` で引数 parse
2. `Commands::Serve(args)` の場合:
   - `ReqwestTepraClient::new(args.creator_base)` を `Arc` で生成
   - `AppState::new_with_template_dir(client, args.template_dir)` を構築
   - 4 つの router builder を `.merge()` し、 `.layer(TraceLayer::new_for_http())`
     を付加して `args.bind` に bind し、 `axum::serve` で起動
3. `Commands::Version` の場合: バージョン文字列を 1 行出力

## 関連 ADR

- `docs/adr/latest/0005-cli-subcommand-split.md` — subcommand 分割の判断
- `docs/adr/latest/0006-http-observability-with-tower-http-tracelayer.md` — TraceLayer 導入の判断
