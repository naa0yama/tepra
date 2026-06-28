# 0002. Per-printer single-worker queue for print jobs

- Status: Accepted
- Date: 2026-06-27
- Deciders: project owner

## Context

The KING JIM TEPRA Creator WebAPI (the .NET module on
`http://localhost:29108`) is single-threaded per physical printer — its
own design dispatches one print job at a time and exposes
`progressOfPrint` / `cancelPrint` / `pauseOfPrint` / `resumeOfPrint`
against a single in-flight job per printer. Posting concurrent `print`
calls to the same printer name does not parallelize the work; behavior
on collision is undocumented and likely returns `PRINT_START_ERROR` or
`PRINTJOB_ACCESS_ERROR`.

At the same time, the UI / API needs to accept print requests freely
without forcing the caller to retry on collision, and surface a useful
"queue position" to operators standing at the printer.

## Decision

Per registered printer, spawn exactly one `tokio::spawn` worker that
consumes a FIFO queue. `POST /api/v1/jobs` always succeeds (provided
parameters validate) and returns `state: "queued"` plus
`queue_position: N`. The worker pops the next `JobId`, calls
`tepra-core::TepraBackend::print`, then polls `job_progress` to terminal,
updating shared `JobMeta` state.

Job state machine:

```
Queued -> Submitting -> Running -> { Completed | Failed | Canceled }
                                   ^
                                   +-- Paused (resume returns to Running)
```

Cancel semantics:

- `DELETE /jobs/{id}` while `Queued`: removed from queue, never
  submitted to .NET.
- While `Submitting`: rejected (short non-cancellable window).
- While `Running` / `Paused`: forwarded to .NET `cancelPrint`.

Printer registration happens at startup from the `getPrinter()` result;
hot-add of printers is deferred to MVP3.

## Consequences

Positive:

- Respects the .NET backend's per-printer single-threaded contract.
- Preserves submission order for operators (FIFO).
- `queue_position` is a meaningful UI affordance.
- Multiple printers run in parallel naturally (one worker each).

Negative:

- Long-running job blocks subsequent queue entries on the same printer
  (acceptable — matches physical reality of a single printer).
- Worker tasks live for the process lifetime; restart loses queue state
  (acceptable for MVP2 in-memory model).
- Restart races between cancel and submission must be handled (cancel
  during `Submitting` window is rejected by design).

## Alternatives Considered

- **Unbounded concurrent submission** — rejected. Violates the .NET
  contract; callers would see opaque `PRINT_START_ERROR` and reimplement
  queueing client-side.
- **Single global worker for all printers** — rejected. Serializes
  unrelated printers, hurting throughput in multi-printer deployments.
- **External queue broker (Redis / NATS)** — rejected. Adds infra
  dependency for what is fundamentally a single-process LAN service.

## History

- 2026-06-27: initial version
