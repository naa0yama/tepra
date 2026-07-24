# tepra Router

`crates/tepra/src/router.rs` が公開する Axum router 群。 全 13 endpoint
の TEPRA Creator `WebAPI` facade と HTML UI を 4 つの builder に分割して
合成する。

## Router builders

- `build_router(client)` — Creator API の read-only facade
  - state: `Arc<dyn TepraClient>`
  - `GET /api/printer` — `list_printers`
  - `GET /api/printer/version` — `version`
  - `GET /api/printer/autoselect` — `autoselect`
  - `GET /api/printer/info/{name}` — `printer_info`
  - `GET /api/printer/onlinestatus/{name}` — `online_status`
  - `GET /api/printer/lwstatus/{name}` — `lw_status`
  - `POST /api/printer/getmargin/{name}` — `get_margin`
  - `GET /api/openapi.json` — `openapi::openapi_json` ( コード由来 OpenAPI 3.1
    ドキュメント配信、`handlers::openapi::ApiDoc::openapi()` を JSON 化 )
- `build_jobs_router(state)` — ジョブ実行系 ( actor 経由 )
  - state: `AppState` ( client + registry )
  - `POST /api/printer/print/{name}` — submit ( queued )
  - `GET /api/printer/tapefeed/{name}?cutflag=<bool>` — テープ送り
  - `GET /api/printer/job/progress/{name}` — 進捗 polling
  - `GET /api/printer/job/info/{name}` — Win32 status bitmask
  - `POST /api/printer/job/control/{name}` — pause / resume / cancel
- `build_templates_router(state)` — テンプレートファイル系
  - `POST /api/printer/template/importframe` — フレーム抽出
  - `GET /api/templates` — `template_dir` 配下の列挙
- `build_ui_router(state)` — HTML UI ( Askama + HTMX )
  - `GET /` — `Redirect::permanent("/ui/")` ( ルートリダイレクト )
  - `GET /ui/` — index
  - `GET /ui/printers/{name}` — 詳細カード
  - `GET /ui/jobs/{printer}/{job_id}` — ジョブカード ( 1s polling 対象 )
  - `GET /ui/api` — API リファレンスページ ( `openapi.json` を in-process で
    view-model 化し DaisyUI accordion で描画。Try it out は既存 `/api/*` route を
    再利用 )

## 合成方法

`crates/tepra-web/src/main.rs` で 4 router を `.merge()` で結合し、
1 つの axum app として `tokio::net::TcpListener` に bind。

## AppState

`crates/tepra/src/state.rs`:

- `client: Arc<dyn TepraClient>` — Creator API 呼出 ( 共有 )
- `registry: Arc<PrinterRegistry>` — per-printer actor lookup
- `template_dir: PathBuf` — テンプレートファイル探索ルート

`AppState` は `Clone` 可で、 axum handler に `State<AppState>` として
注入する。

## エラー写像

Creator API 呼出失敗は handler 層で `StatusCode::BAD_GATEWAY` (502) に
写像 ( `printers.rs::err_502` 参照 )。

## OpenAPI ドキュメント生成

OpenAPI はコード由来で生成し、手書き spec を持たない ( drift 回避 )。責務分割は
ADR 0010 に従う:

- **`tepra-core`**: DTO のデータ形状 ( スキーマ )。`#[cfg_attr(feature = "schema",
  derive(utoipa::ToSchema))]` を付与し `schema` feature の下でのみ `utoipa` に依存
  ( 詳細は `tepra-core-tepra-client.md` )。
- **`tepra`** ( 本 crate ): HTTP operation metadata。各 handler に
  `#[utoipa::path]` を付与し `handlers::openapi::ApiDoc` ( `#[derive(OpenApi)]` )
  に集約、`GET /api/openapi.json` で配信する。`tepra-core` を
  `features = ["schema"]` で有効化。
- 配信される paths は router の実 route と 1:1 対応する ( 統合テスト
  `handlers_openapi.rs` が全 path と主要 schema の存在を assert )。
