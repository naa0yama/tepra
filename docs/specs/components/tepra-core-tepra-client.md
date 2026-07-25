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
  ( res item shape: `ImportFrameItem { column, title, attribute }` —
  詳細は「Merge-print orchestration」節参照 )
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

## Merge-print orchestration ( `tepra` crate )

`.lw1` テンプレートに CSV データを流し込んで印刷する `POST
/api/rest/merge-print/{printer}` ( `tepra-router.md` 参照 ) を支える型と純粋関数。
`tepra-core` の DTO ではなく `crates/tepra/src/merge.rs` /
`crates/tepra/src/handlers/merge_print.rs` に置く ( orchestration とその型は
web crate の関心事、`TepraClient` trait とは別レイヤ )。

### 型

- `MergePrintRequest { template: String, rows: Vec<Vec<MergeField>>, serial:
  Option<SerialSpec>, #[serde(flatten)] overrides: MergePrintOverrides }`
  ( `handlers/merge_print.rs` ) — `rows` の外側 = テープ ( ラベル ) 単位、
  内側 = そのテープの `{title, value}` フィールド配列
- `MergeField { title: String, value: String }` ( `merge.rs` ) — `title` は
  `ImportFrameItem::title` と一致する列タイトル ( 画面表示ヘッダ )
- `SerialSpec { title, start: i64, count: u32, step: i64, pad: u8 }`
  ( `merge.rs` ) — 連番生成スペック。ネイティブ WebAPI が無いため
  `expand_serial` がサーバ側で流し込み行を生成する
- `MergePrintOverrides` ( `merge.rs` ) — `copies` / `density` / `tape_cut` /
  `half_cut` / `half_cut_separate` / `print_speed` / `margin_left_right` を
  すべて `Option` で保持、未指定は wire 既定 ( `merge_print_parameter` 参照 )

### 純粋関数 ( I/O 無し、単体テスト対象 )

- `build_merge_csv(frames: &[ImportFrameItem], rows: &[Vec<MergeField>],
  encoding: CsvEncoding) -> anyhow::Result<Vec<u8>>` — `frames` の `column`
  順でヘッダ無し RFC4180 CSV を組み立てる。値は各行 ( テープ ) の `title` で
  該当枠を引いて解決 ( 欠損 → 空 )。`frames` の重複 `title`、または行内の
  未知/重複 `title` は `Err` ( handler で 400 に写像 )。`CsvEncoding::{Utf8,
  ShiftJis}` ( `ShiftJis` は `encoding_rs` の CP932、既定は `Utf8` )
- `expand_serial(spec: &SerialSpec) -> Vec<MergeField>` — `start` から `step`
  刻みで `count` 個、`pad` 桁 0 埋めした値を `title` の `MergeField` として生成
- `merge_print_parameter(overrides: &MergePrintOverrides) -> PrintParameter`
  — SDK `defaultPrintParameter` ( `tepraprint.js` ) 由来の wire 既定値に
  `overrides` を適用して `PrintParameter` を構築

### DTO 是正: `ImportFrameItem`

`crates/tepra-core/src/dto/template.rs` の `ImportFrameItem` は
`{ column: String, title: String, attribute: ImportFrameAttribute }` が正
( 旧 `id`/`width`/`height` は一次ソース `tepraprint.js` の importframe レスポンス
と不一致だったため是正済み )。`column` はセル参照 ( `A1`/`B2` 等、CSV↔枠の
バインドキー )、`title` は Creator 作成画面で入力した列タイトル ( UI 表示用 )。
両者は別物で、`build_merge_csv` は `title` で解決し `column` 順で出力する。

### Lister 是正: `.lw1` テンプレート列挙

`crates/tepra/src/templates.rs` の一覧関数は `is_lbl` ( `.lbl` のみ判定 ) から
`is_template` に改名し、`.lw1` / `.lbl` を case-insensitive で受理するよう
是正済み ( 実テンプレファイルの拡張子は `.lw1` だが旧実装は `.lbl` のみを
対象としており、`GET /api/rest/templates` に対象テンプレが出てこなかった )。

## OpenAPI スキーマ導出 ( `schema` feature )

`tepra` ( web ) の API リファレンスページ / `openapi.json` が使う DTO スキーマ
は、DTO 定義そのものと同じ場所 = `tepra-core` に置く。 これにより cli など別
front-end も同じスキーマ導出を再利用できる ( ADR 0010 )。

- `dto/` の Request/Response 型に
  `#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]` を付与
- 各 pub field に `///` doc comment を付与する。`utoipa` は struct doc comment
  を schema `description`、field doc comment を各 property の `description` に
  出力するため、これらが `openapi.json` 経由で API リファレンスページの
  プロパティ表 ( field / 型 / 必須 / 説明 ) の「説明」列を埋める。doc comment
  自体は feature gate 不要 ( `schema` feature 有効時のみ utoipa が拾う )
- `utoipa` 依存は `schema = ["dep:utoipa"]` feature の下に閉じ込め、default build
  には一切 leak しない ( `cargo tree` で default に utoipa が現れないことを確認済み )
- HTTP operation metadata ( `#[utoipa::path]` / `#[derive(OpenApi)]` ) は
  `tepra-core` には置かず web crate 側に留める。 責務分割の詳細は
  `tepra-router.md` の「OpenAPI ドキュメント生成」節を参照

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
- `docs/adr/latest/0010-openapi-schema-derivation-in-core-behind-feature-gate.md`
  — `schema` feature 境界の決定記録
- `crates/tepra-core/src/dto/` — Request/Response DTO 定義
