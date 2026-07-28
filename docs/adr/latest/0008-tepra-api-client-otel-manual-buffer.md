# 0008. TEPRA WebAPI client OTel 計装は手動 buffer 方式を採用

- Status: Accepted
- Date: 2026-07-03
- Deciders: naa0yama

## Context

`ReqwestTepraClient` に OTel HTTP client semconv 準拠の req/res body 捕捉を追加する際、
実装アプローチとして以下の選択肢を検討した。

- **A. 手動 serialize + 手動 read**: 各メソッド内で `serde_json::to_vec` / `resp.bytes()`
  を使い、body を明示的に buffer 化して span attribute と tracing::debug! log に記録
- **B. reqwest-middleware + reqwest-tracing**: HTTP client middleware crate を導入し、
  OTel span 生成と属性記録を middleware 層に委譲
- **C. tower::Service 化**: reqwest を `tower::Service` に wrap し、
  server 側と同じ `tower-http::TraceLayer` を流用

観測目的は「デバッグ再現 (curl 再現可能な粒度)」。
TEPRA Creator WebAPI は localhost-only の 13 endpoint 固定。

## Decision

**Approach A (手動 buffer)** を採用する。

各 HTTP helper メソッド (`get_json`, `get_query_json`, `get_query_empty`,
`post_json`, `post_empty`) 内で body を明示的に buffer 化し、
`Span::current().record` と `tracing::debug!` / `tracing::warn!` で
span attribute と log field を発火する。

送信は `RequestBuilder::build()` → `Request` → `Client::execute()` の二段構えとし、
`Request::headers()` から reqwest 内部付与の header (content-length 等) も捕捉する。

## Consequences

**Positive:**

- 追加 dep は `tracing-test` (dev-only) のみ。本番 binary サイズに影響なし
- `WireMock` ベースの既存 integration test 構造をそのまま踏襲できる
- Layer 1 (Application Code) レイヤで BLOCK/REDACT header 判定が完結し、
  下流 layer (Collector/Backend) に依存しない
- endpoint 毎のカスタム挙動 (特定 endpoint のみ body をスキップ等) が容易

**Negative:**

- 5 メソッド全てに buffer 化 + record 呼出しが入り、メソッド本体が長くなる。
  private helper (`record_request_headers`, `record_response_headers`) で軽減
- `reqwest-middleware` を後から導入する場合、今回書いたロジックの一部を
  middleware に移植し直す必要がある

## Alternatives Considered

**B. reqwest-middleware + reqwest-tracing**

- Rust HTTP client OTel 計装の標準的候補
- ただし `reqwest-tracing` は body 記録に対応しておらず、body 全量捕捉は
  自作 middleware が必要。結局 Approach A と同程度の実装量になる
- 抽象化層が増え、デバッグ時の tracing が複雑になる
- 却下: コスト対効果で劣る

**C. tower::Service 化**

- reqwest は `tower::Service<Request>` を直接実装しておらず、変換 layer が必要
- 現状 HTTP のみ、gRPC 等の追加計画もないため過剰設計
- 却下: 現在の問題に対してスコープが大きすぎる

## History

- 2026-07-03: initial version
