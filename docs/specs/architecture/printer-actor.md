# Printer Actor Architecture

**Not wired into the production call path.** The `actor` / `registry`
modules below remain in the tree and are exercised by
`crates/tepra/tests/actor_*.rs`, but no handler routes through them.
Production job submission uses the synchronous fire-and-record path from
ADR 0011 (Ephemeral in-memory job store, supersedes ADR 0002 / ADR 0004):
handlers (`crates/tepra/src/handlers/jobs.rs`,
`crates/tepra/src/handlers/merge_print.rs`) call `state.client.print()`
directly and await the result — no queue, no per-printer worker task.
`AppState.registry` (`crates/tepra/src/state.rs`) is retained but referenced
only by `AppState`'s `Debug` impl; nothing reads or spawns through it at
runtime.

The `queued` job state and `queue_position` response field described below
are **not implemented**. `POST /api/printer/print/{name}` and the
`merge_print` orchestration return `Accepted { jobid }` or a failure
directly from the synchronous `client.print()` call; there is no queued
intermediate state.

## Original actor design (historical, test-only)

`tepra` のジョブ実行層は per-printer の actor pattern で構成する設計だった。
KING JIM TEPRA Creator `WebAPI` ( `http://localhost:29108/api/printer` )
は物理プリンタ毎 single in-flight job なため、 1 プリンタ = 1 worker task
を型で表現する狙いだった。

### 構成要素

- `PrinterRegistry` ( `crates/tepra/src/actor/registry.rs` )
  - `DashMap<String, Arc<PrinterHandle>>` でプリンタ名 → handle を保持
  - `get_or_spawn(name)` で lazy spawn ( 初回アクセス時に 1 task 生成 )
  - `shutdown_all()` で全 actor に `Msg::Shutdown` 送信
- `PrinterActor` ( `crates/tepra/src/actor/printer.rs` )
  - `tokio::spawn` で起動する 1 task = 1 worker
  - `mpsc::Receiver<Msg>` でメッセージ受信、状態は task 内 `WorkerState`
    に閉じ込め ( 外部から参照不可 )
  - FIFO 順に `queue: VecDeque<(JobId, PrintRequest)>` を処理
  - in-flight 1 件 + 過去ジョブ状態を `HashMap<JobId, JobState>` に保持
- `PrinterHandle`
  - `mpsc::Sender<Msg>` の wrapper、 `Clone` 可
  - `Submit` / `Cancel` / `Status` / `CurrentJob` / `Shutdown` を提供
  - 全レスポンスは `oneshot::Sender` で同期的に返却

### メッセージフロー (未配線)

```
HTTP request
  -> axum handler ( handlers/jobs.rs )
  -> AppState.registry.get_or_spawn(name) -> Arc<PrinterHandle>
  -> handle.submit(req) -> mpsc::Sender<Msg::Submit>
       worker loop ( single task ):
         pop next job -> TepraClient::print -> poll progress
                      -> update JobState ( Completed | Cancelled | Failed )
  -> oneshot::Receiver で JobId 返却
```

### 不変条件 (未配線コードの設計意図)

- 1 プリンタにつき 1 task のみ存在 ( `DashMap::entry().or_insert_with` で
  保証 )
- 状態 mutate は worker task 内のみ。 handler は message 越しでしか触れない
- shutdown は graceful: 受信済みジョブを完走してから task 終了

## 現行方式の残存リスク

Creator WebAPI 仕様 (`docs/specs/external/tepra-creator-webapi.md`) には
同一プリンターへの同時リクエストを直列化する契約が存在しない。actor 方式を
採用せず handler が `client.print()` を同期・直接呼び出す現行方式は、
「実運用で同一プリンターへの同時 HTTP リクエストが発生しない」という前提に
暗黙に依存する。この前提が崩れる (同一プリンターへ並行 print リクエストが
来る) 場合、Creator API 側の挙動は未定義であり `PRINT_START_ERROR` /
`PRINTJOB_ACCESS_ERROR` 等の予期しない失敗を招く可能性がある。actor/queue
方式への再配線、またはリクエスト単位の排他ロック導入は、この前提が崩れた
時点で再検討する。

## 関連 ADR

- `docs/adr/latest/0002-per-printer-single-worker-queue.md` — Superseded
  by ADR 0011 (queue 方針。現在は不使用)
- `docs/adr/latest/0004-printer-actor-pattern.md` — Superseded by
  ADR 0011 (actor 採用判断。現在は不使用)
- `docs/adr/latest/0011-ephemeral-in-memory-job-store.md` — 現行の
  同期 fire-and-record 方式を規定する ADR
