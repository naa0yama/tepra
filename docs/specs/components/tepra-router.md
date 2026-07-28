# tepra Router

`crates/tepra/src/router.rs` が公開する Axum router 群。 TEPRA Creator
`WebAPI` facade (全 13 endpoint) と、プログラム独自 REST・HTML UI を
5 つの builder に分割して合成する。

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
  - `GET /api/rest/templates` — `template_dir` 配下の列挙 ( 旧 `GET /api/templates`
    から `/api/rest/` 名前空間へ移設。公式 Creator WebAPI facade
    (`/api/printer/*`) とプログラム独自 REST の path prefix 分離のため )
  - `GET /api/rest/templates/preview` — `.lw1` 先頭 BMP プレビュー切出
    ( `?path=<rel>`、`template_dir` 配下に正規化して path traversal 防止、
    無回転 ( テープ印刷方向と一致 ) の元 BMP を `image/bmp` で返す )
- `build_merge_router(state)` — 流し込み印刷 ( merge print )
  - state: `AppState`
  - `POST /api/rest/merge-print/{printer}` — テンプレ + CSV 流し込み印刷の
    orchestration ( テンプレ読込 → `import_frame` → `sort_frames_by_column`
    で column セル参照順に正規化 → CSV 組立 → `print` )。正規化は
    `fetch_template_and_frames` が担い、CSV 組立 ( `merge_print` ) と UI 描画
    ( `print_frames` ) の両経路が同一の正規化済み順序を共有する。
    req: `MergePrintRequest { template, rows: Vec<Vec<MergeField>>,
    serial: Option<SerialSpec>, #[serde(flatten)] overrides:
    MergePrintOverrides }` ( `rows` の外側 = テープ単位、内側 =
    `{title, value}` フィールド配列 )。res: 既存 `PrintResponse { result,
    jobid }`。テンプレ未存在 → 404、未知/重複 `title` または `rows`/`serial`
    件数不整合 → 400、Creator エラー → 502。型・純粋関数の詳細は
    `tepra-core-tepra-client.md` の「Merge-print orchestration」節を参照。
    `client.print` の Ok/Err 結果は `AppState.jobs` (`JobStore`) へ 1 箇所で
    記録 ( UI `POST /ui/print/{printer}` と本 REST route の両方をカバー。
    詳細は ADR 0011 参照 )
- `build_ui_router(state)` — HTML UI ( Askama + HTMX )
  - `GET /` — `Redirect::permanent("/ui/")` ( ルートリダイレクト )
  - `GET /ui/` — index
  - `GET /ui/printers/{name}/status-card` — ステータスカード ( HTMX lazy-load 対象 )
  - `GET /ui/jobs?page=N` — ジョブ履歴一覧 ( `JobStore::page` で新しい順に
    サーバ側スライス。20 件/page、`page` 未指定は 1、範囲外はクランプ。
    最大 100 件保持のため最大 5 page。詳細は `askama-templates.md` の
    `pages/jobs.html` 節を参照 )
  - `GET /ui/jobs/{printer}/{job_id}` — ジョブカード ( 1s polling 対象 )
  - `POST /ui/jobs/{printer}/{job_id}/cancel` — ジョブキャンセル
    ( `job_control(control=3)` → `job_progress` 再取得 → `JobCardTemplate`
    描画。raw API `POST /api/printer/job/control/{name}` は JSON body 前提で
    htmx の form-enc submit と噛み合わないため UI 専用 route として新設 )
  - `GET /ui/api` — API リファレンスページ ( `openapi.json` を in-process で
    view-model 化し DaisyUI accordion で描画。Try it out は既存 `/api/*` route を
    再利用 )
  - `GET /ui/print` — 流し込み印刷ページ ( テンプレプレビュー + テープ入力 +
    printer 情報パネル )。`?from={record_id}` 指定時は `JobStore` から
    該当ジョブを検索し、保存済み `MergePrintRequest` の値をサーバ側で
    インラインレンダー ( `value=`/`checked`/selected option ) して
    再印刷フォームとして再現する ( 再印刷ボタンの遷移先。詳細は
    `askama-templates.md` の `pages/print.html` 節を参照 )
  - `GET /ui/print/frames` — テープ入力 partial ( テンプレ選択で htmx swap、
    importframe 由来の frame 一覧を返す )
  - `POST /ui/print/{printer}` — 印刷送信 ( 共有 `merge_print()` orchestration
    を呼び出し `JobCardTemplate` を返す )
  - `GET /ui/print/{printer}/panel` — printer 情報パネル ( onlinestatus +
    lwstatus + getmargin を集約。online/offline/busy/device-error/接続不可の
    5-way 表示マッピングは `askama-templates.md` の「Printer status display
    mapping」節を参照。ここでは複製しない )

## 合成方法

`crates/tepra-web/src/main.rs` で 5 router を `.merge()` で結合し、
1 つの axum app として `tokio::net::TcpListener` に bind。

## AppState

`crates/tepra/src/state.rs`:

- `client: Arc<dyn TepraClient>` — Creator API 呼出 ( 共有 )
- `registry: Arc<PrinterRegistry>` — per-printer actor lookup
- `template_dir: PathBuf` — テンプレートファイル探索ルート
- `jobs: Arc<JobStore>` — 印刷ジョブ履歴 ( in-memory、上限 100 件。
  ephemeral — プロセス再起動で消失。ADR 0011 参照 )

`AppState` は `Clone` 可で、 axum handler に `State<AppState>` として
注入する。

## エラー写像

Creator API 呼出失敗は handler 層で `StatusCode::BAD_GATEWAY` (502) に
写像 ( `printers.rs::err_502` 参照 )。

`GET /ui/print/{printer}/panel` / `GET /ui/printers/{name}/status-card` は
502 に丸めず、lwstatus 404 (busy) / device error / offline / 接続不可を
個別表示に分岐する ( 5-way マッピングは `askama-templates.md` の「Printer
status display mapping」節を参照。詳細をここへ複製しない )。

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
