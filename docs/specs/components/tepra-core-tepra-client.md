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

## Observability ( OTel client span )

`ReqwestTepraClient` の 13 caller は全て `#[instrument]` を付与し、
OTel HTTP client semantic conventions 1.23+ 準拠の CLIENT span を emit する。

- span name は静的リテラル `"{METHOD} {url.template}"` 形式で低カーディナリティ
  ( trace UI 上で endpoint 別に集約可能 )
- helper ( `get_json` / `get_json_query` / `get_query_empty` / `post_json` /
  `post_empty` ) には `#[instrument]` を付与しない。 helper 側は
  `Span::current().record(...)` で caller span に属性を追記する
  ( bare `"GET"` / `"POST"` の inner span を emit させない )
- caller span 属性:
  - `otel.kind = "CLIENT"`
  - `http.request.method` = 静的 `"GET"` / `"POST"`
  - `url.template` = 静的 template ( 例: `/api/printer/info/{name}` )
  - `server.address` / `url.scheme` = client 設定値
  - `url.full` = 展開後の実 URL ( helper record )
  - `http.response.status_code` / `http.response.body.size` = helper record

span name 一覧 ( 13 caller ):

- `GET /api/printer` — `list_printers`
- `GET /api/printer/version` — `version`
- `GET /api/printer/autoselect` — `autoselect`
- `GET /api/printer/info/{name}` — `printer_info`
- `GET /api/printer/onlinestatus/{name}` — `online_status`
- `GET /api/printer/lwstatus/{name}` — `lw_status`
- `GET /api/printer/tapefeed/{name}` — `tapefeed`
- `GET /api/printer/job/progress/{name}` — `job_progress`
- `GET /api/printer/job/info/{name}` — `job_info`
- `POST /api/printer/print/{name}` — `print`
- `POST /api/printer/job/control/{name}` — `job_control`
- `POST /api/printer/template/importframe` — `import_frame`
- `POST /api/printer/getmargin/{name}` — `get_margin`

実装メモ:

- 動的 path template ( `{name}` を含むリテラル ) は clippy
  `literal_string_with_formatting_args` を誤発火するため、
  `concat!("GET ", "/api/printer/info/{name}")` でリテラル分割する
- caller span の record 期待は `tests/client_span_name.rs` で検証。
  wiremock + tracing-subscriber の custom Layer で 13 endpoint の
  span name / `url.template` / `otel.kind` / `http.request.method` を
  assert し、bare `"GET"` / `"POST"` span が emit されないことも保証する

## 関連

- `docs/specs/architecture/otel-instrumentation.md` — 全体計装方針
- `docs/specs/external/tepra-creator-webapi.md` — Creator API の生仕様
- `crates/tepra-core/src/dto/` — Request/Response DTO 定義
