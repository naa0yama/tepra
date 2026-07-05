# OTel Instrumentation

本プロジェクト全体の OpenTelemetry ( `tracing` + `tracing-opentelemetry` +
`opentelemetry-otlp` ) 計装方針を記載する。個別コンポーネント側の実装
詳細は `docs/specs/components/` を参照。

## Trace signal

### HTTP server span ( inbound )

- `tower_http::trace::TraceLayer::new_for_http()` を router 最上位に配置
  ( ADR-0006 )
- 各 axum handler に `#[instrument(name = "handler.<fn>", skip_all,
  fields(...))]` を付与し child span を emit
- 属性は OTel HTTP server semantic conventions 準拠:
  - `http.request.method` / `http.route` / `http.response.status_code` /
    `url.scheme`

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

## 命名規則

- OTel semconv 定義済 span ( HTTP / DB / RPC ) は semconv 準拠
  ( 例: `GET /users/{id}` )
- application 固有 span は `<component>.<operation>` 形式
  ( 例: `handler.list_printers` / `printer_actor.run` )
- 動的値 ( path parameter 実値、user id 等 ) は span name に含めず属性に載せる

## 関連

- ADR-0006 — HTTP observability with tower-http TraceLayer
- `docs/specs/components/tepra-core-tepra-client.md` — client span 詳細
