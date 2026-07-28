# OTel Instrumentation

本プロジェクト全体の OpenTelemetry ( `tracing` + `tracing-opentelemetry` +
`opentelemetry-otlp` ) 計装方針を記載する。個別コンポーネント側の実装
詳細は `docs/specs/components/` を参照。

## Lifecycle

### `TelemetryGuard` と shutdown 順序

- `tepra_core::otel::init_telemetry(service_name, git_hash)` が subscriber と
  3 provider ( `SdkTracerProvider` / `SdkMeterProvider` / `SdkLoggerProvider` )
  の初期化を担う ( `crates/tepra-core/src/otel/mod.rs` )
- 戻り値 `TelemetryGuard` は process 生存期間中 provider を保持する。
  明示 `shutdown()` ( async ) を呼ぶまで `Drop` は「未 shutdown 検出時の
  警告のみ」に留める ( blocking shutdown をそのまま `Drop` で呼ぶと tokio
  executor をブロックするため )
- shutdown 順序: tracer ( `shutdown()` ) → meter ( `force_flush()` →
  `shutdown()` ) → logger ( `shutdown()` )。blocking 呼び出しは
  `std::thread::spawn` 上で実行し、5 秒 timeout 付きで完了を待つ
- 複数回 `shutdown()` を呼んでも 2 回目以降は no-op ( `AtomicBool` で guard )

### `OTEL_EXPORTER_OTLP_ENDPOINT` によるランタイム activation

- `OTEL_EXPORTER_OTLP_ENDPOINT` が設定 ( 非空 ) の場合のみ OTLP exporter
  ( tracer / meter / logger ) を構築・有効化する。未設定時は stderr fmt
  layer のみで動作し `TelemetryGuard::Disabled` を返す ( dev/test 用の
  フォールバック )
- W3C Trace Context propagator ( `TraceContextPropagator` ) は
  endpoint 有無に関わらず常時登録し、incoming `traceparent` の抽出を可能
  にする
- endpoint 文字列自体は SDK に読ませ、exporter builder 側で
  `.with_endpoint()` を明示しない ( SDK が `{base}/v1/{signal}` を自動付与
  する env var 経路を採用 )

### Resource attributes

- `service.name` ( 呼び出し元 binary から注入、`OTEL_SERVICE_NAME` env
  var で上書き可 ) / `service.version` ( `CARGO_PKG_VERSION` ) /
  `vcs.ref.head.revision` ( `GIT_HASH`、後述 build.rs 由来 )
- `service.instance.id` ( `{hostname}-{pid}` ) / `host.name`
  ( `gethostname` ) / `host.arch` / `os.type` / `process.pid` /
  `process.executable.name` / `process.runtime.name` ( `"rustc"` ) /
  `process.runtime.version`
- `vcs.ref.head.revision` ( `GIT_HASH` ) は `crates/tepra-web/build.rs`
  が `git rev-parse --short=12 HEAD` を `cargo:rustc-env=GIT_HASH=...` で
  埋め込み、`env!("GIT_HASH")` として `init_telemetry()` に渡す

## Trace signal

### HTTP server span ( inbound )

- `tower_http::trace::TraceLayer::new_for_http()` を router 最上位に配置
  ( ADR-0006 )
- 各 axum handler に `#[instrument(name = "handler.<fn>", skip_all,
  fields(...))]` を付与し child span を emit
- 属性は OTel HTTP server semantic conventions 準拠:
  - `http.request.method` / `http.route` / `http.response.status_code` /
    `url.scheme`
- server span 生成は `OtelHttpServerMakeSpan` ( `crates/tepra-web/src/trace.rs` )
  が担い、`opentelemetry_http::HeaderExtractor` + 登録済み propagator で
  incoming `traceparent` header を抽出、`Span::set_parent(...)` により
  W3C context propagation ( 受信側 ) を実現する
- 5xx 応答時は `OtelOnResponse` が `span.set_status(Status::Error)` +
  `error.type` 属性を record する

### HTTP client span ( outbound to TEPRA Creator API )

- `ReqwestTepraClient` の 13 caller ( `TepraClient` trait 実装 ) それぞれに
  `#[instrument]` を付与
- span name は静的リテラル `"{METHOD} {url.template}"` 形式
  ( 低カーディナリティ、path parameter 展開後の実 URL は含めない )
- `url.template` 属性を明示付与 ( OTel HTTP client semantic conventions
  1.23+ 準拠 )
- helper ( `get_json` / `get_json_query` / `get_query_empty` / `post_json` /
  `post_empty` ) 側には `#[instrument]` を付与しない。属性 record は
  `Span::current().record(...)` 経由で caller span に伝播する
- 属性一覧:
  - `otel.kind = "CLIENT"`
  - `http.request.method` = 静的 `"GET"` / `"POST"`
  - `url.template` = 静的 template ( 例: `/api/printer/info/{name}` )
  - `url.full` = 展開後の実 URL ( helper 側で record )
  - `server.address` / `url.scheme` = client 設定値
  - `http.response.status_code` / `http.response.body.size` = helper 側
    で record

span name 一覧は `docs/specs/components/tepra-core-tepra-client.md` の
Observability セクション参照。

## Metric signal

- `http.client.request.duration` — `ReqwestTepraClient` が emit。attribute
  cardinality 抑制のため `url.template` は **含めない** ( `server.address`
  / `server.port` / `http.request.method` / `url.scheme` / `error.type`
  のみ )
- `http.server.request.duration` — `server_metrics_mw` middleware が
  `method` / `route` を populate
- HTTP latency histogram ( client / server 共通 ) は SDK default bucket
  ではなく `SdkMeterProvider::builder().with_view(...)` で semconv 推奨の
  明示 bucket boundaries ( 14 段階、`[0.005, ..., 10.0]` 秒 ) を指定する
  ( `crates/tepra-core/src/otel/meter.rs` の `http_latency_view` )
- process metrics ( `process.cpu.utilization` / `process.memory.usage` /
  `process.thread.count` / `process.uptime` 等 ) は `sysinfo` 直接実装
  ( `tepra-core/src/otel/metrics/process.rs` ) で登録し、`Meters::new()` /
  `set_meter_provider` 呼び出し順序を守ることで OTLP export まで到達する
  ( 順序が逆だと登録済みでも export されない )
- `error.type` は semconv named value を優先する優先順位で分岐する
  ( `crates/tepra-core/src/client/reqwest_client.rs` ): `is_timeout()` →
  `"timeout"` / `is_connect()` → `"connection"` / `is_request()` →
  `"request_build"` / それ以外 → `"_OTHER"`。HTTP status error は
  `status.as_u16().to_string()` を維持する

## 命名規則

- OTel semconv 定義済 span ( HTTP / DB / RPC ) は semconv 準拠
  ( 例: `GET /users/{id}` )
- application 固有 span は `<component>.<operation>` 形式
  ( 例: `handler.list_printers` / `actor.worker.run` )
- 動的値 ( path parameter 実値、user id 等 ) は span name に含めず属性に載せる

## 関連

- ADR-0006 — HTTP observability with tower-http TraceLayer
- `docs/specs/components/tepra-core-tepra-client.md` — client span 詳細
