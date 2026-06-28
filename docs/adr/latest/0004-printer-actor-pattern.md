# 0004. Printer actor pattern for FIFO single-worker queue

- Status: Accepted
- Date: 2026-06-28
- Deciders: project owner

## Context

ADR 0002 commits to per-printer single-worker FIFO queues. That decision
leaves the implementation pattern open. Several options exist for a
multi-printer Tokio service that must hold per-printer mutable state
(in-flight job, cancel handle, lwstatus cache) and serialize work
strictly per printer:

- A **thin facade** that keeps queue maps inside axum handlers and uses
  raw `tokio::sync::Mutex` for per-printer state.
- A **layered service** with a `JobService` orchestrating queue
  dispatchers, called from handlers and backed by a `TepraClient`
  trait.
- An **actor pattern** where each printer is owned by exactly one
  `tokio::spawn` task that consumes messages over an mpsc channel.

The defining constraint is that "exactly one worker per printer" must
hold across all code paths — submission, progress polling, cancel — and
must not be accidentally violated when new handlers are added.

## Decision

Adopt the actor pattern. Each printer is represented by a
`PrinterActor` running in a dedicated `tokio::spawn` task. The actor
owns all per-printer state (`VecDeque<Job>`, `Option<JobState>` for the
in-flight job, lwstatus cache, current cancel token). Handlers
interact with the actor only by sending `Msg` enum variants over the
actor's `mpsc::Sender`; replies come back over per-message
`oneshot::Sender`s. A `PrinterRegistry` (`DashMap<String,
PrinterHandle>`) provides handle lookup by printer name.

```
HTTP handler -> PrinterRegistry::actor(name) -> PrinterHandle
                                                    |
                                                    | mpsc::Sender<Msg>
                                                    v
                                               PrinterActor (1 task)
                                                    |
                                                    v
                                               TepraClient (trait)
```

The single-task invariant is expressed in types: actor state is owned
by the task and only mutated inside its loop. The Layered alternative
would have required additional `Mutex` discipline to enforce the same
property.

## Consequences

Positive:

- The per-printer single-worker constraint is structural, not
  conventional. Adding a new handler cannot bypass the invariant
  without explicitly constructing a second actor.
- Cancel / pause / resume map naturally to `Msg::Cancel(JobId)` etc. —
  the actor inspects in-flight state without a separate lock.
- Progress polling reads cached state from the actor without contending
  with the print loop.
- `MockTepraClient` plugs in via the trait; actor logic is
  trait-agnostic and unit-testable without a real backend.

Negative:

- Boilerplate: each handler operation needs a `Msg` variant and a
  `oneshot::Sender` for the reply, increasing line count vs raw
  function calls in a Layered design.
- Backpressure: if the mpsc channel fills, handlers block. We pick
  bounded channels (`mpsc::channel(64)`) and treat overflow as 503.
- Panics inside the actor task terminate the worker; supervision (auto
  respawn) is deferred to MVP3.

## Alternatives Considered

- **Thin facade with per-printer Mutex** — rejected. Easy to violate
  the single-worker invariant when adding handlers; cancel handling
  requires extra channels anyway, eroding the simplicity advantage.
- **Layered service** — rejected. Equally testable but requires manual
  per-printer locking to enforce single-worker behavior; the type
  system does not encode the constraint.
- **External actor framework (actix / xtra)** — rejected. Adds a
  dependency for a pattern small enough to express in ~100 lines of
  `tokio::sync` primitives.

## History

- 2026-06-28: initial version
