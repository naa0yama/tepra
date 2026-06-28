# tepra-core `TepraClient`

`crates/tepra-core/src/client/` が公開する TEPRA Creator `WebAPI` 抽象。
全 13 endpoint を 1 trait にまとめ、本番 `ReqwestTepraClient` と
テスト用 `MockTepraClient` の 2 実装を提供する。

## Trait

`pub trait TepraClient: Send + Sync` ( `client/traits.rs` )

- `list_printers` — `GET /api/printer`
- `version` — `GET /api/printer/version`
- `autoselect` — `GET /api/printer/autoselect`
- `printer_info(name)` — `GET /api/printer/info/{name}`
- `online_status(name)` — `GET /api/printer/onlinestatus/{name}`
- `lw_status(name)` — `GET /api/printer/lwstatus/{name}`
- `print(name, req)` — `POST /api/printer/print/{name}`
- `tapefeed(name, cutflag)` — `GET /api/printer/tapefeed/{name}?cutflag=<bool>`
- `job_progress(name, jobid)` — `GET /api/printer/job/progress/{name}?jobid=N`
- `job_info(name, jobid)` — `GET /api/printer/job/info/{name}?jobid=N`
- `job_control(name, req)` — `POST /api/printer/job/control/{name}`
- `import_frame(req)` — `POST /api/printer/template/importframe`
- `get_margin(name, req)` — `POST /api/printer/getmargin/{name}`

`async_trait` を使用。 `Arc<dyn TepraClient>` で `AppState` に注入。

## 実装

- `ReqwestTepraClient` ( `client/reqwest_client.rs` ) — `reqwest::Client`
  ベース。 `base_url` を constructor で受け取り、 default は
  `http://localhost:29108`
- `MockTepraClient` ( `client/mock.rs` ) — 単体テスト用。 `MockCall` enum
  で呼出履歴を記録、 fixture レスポンスを返す

## 仕様逸脱メモ

- `tapefeed` は GET ( spec 上は `POST` と記載されていた )
  - 根拠: 公式 SDK `tepraprint.js` L990 が plain `fetch` 呼出
    ( default GET ) で
    `${uri}/tapefeed/${name}?cutflag=${cutFlag}` を発行
  - 採用: `tapefeed(&self, name: &str, cutflag: bool)` シグネチャ。
    `cutflag` は Rust の `Display` ( `"true"` / `"false"` ) でエンコード、
    JS `Boolean.toString()` 互換
  - 影響: `MockCall::Tapefeed(String, bool)` も同じ shape

## エラー型

`TepraError` ( `error.rs` ):

- `Transport { source }` — `reqwest` の send 失敗
- `Parse { source }` — JSON deserialize 失敗
- Creator API の errcode は今後 `dto::error` で扱う方針

## 関連

- `docs/specs/external/tepra-creator-webapi.md` — Creator API の生仕様
- `crates/tepra-core/src/dto/` — Request/Response DTO 定義
