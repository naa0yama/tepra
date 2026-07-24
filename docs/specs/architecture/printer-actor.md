# Printer Actor Architecture

`tepra` のジョブ実行層は per-printer の actor pattern で構成。
KING JIM TEPRA Creator `WebAPI` ( `http://localhost:29108/api/printer` )
は物理プリンタ毎 single in-flight job なため、 1 プリンタ = 1 worker task
を型で表現する。

## 構成要素

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

## メッセージフロー

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

## 不変条件

- 1 プリンタにつき 1 task のみ存在 ( `DashMap::entry().or_insert_with` で
  保証 )
- 状態 mutate は worker task 内のみ。 handler は message 越しでしか触れない
- shutdown は graceful: 受信済みジョブを完走してから task 終了

## 関連 ADR

- `docs/adr/latest/0002-per-printer-single-worker-queue.md` ( queue 方針 )
- `docs/adr/latest/0004-printer-actor-pattern.md` ( actor 採用判断 )
