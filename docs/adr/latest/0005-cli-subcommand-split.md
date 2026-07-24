# 0005. CLI subcommand split for Windows deployment

- Status: Accepted
- Date: 2026-06-28
- Deciders: project owner

## Context

The KING JIM TEPRA Creator WebAPI is a Windows-only .NET service. The
production deployment target is therefore a Windows host running both
the Creator backend and `tepra`. Two user-facing affordances are
required on Windows that do not apply to the headless Linux
development build:

- A **system tray icon** so an operator can confirm the service is
  running and reach the local UI without opening a terminal.
- **Windows service registration** so the binary starts on boot
  without an interactive logon.

Linux/dev builds need neither and should not pull in Windows-only
crates (`tray-icon`, `windows-service`) for cross-platform CI.

## Decision

Ship a single binary `tepra` with clap-derived subcommands. Each
subcommand is gated by `#[cfg(...)]` so the binary remains
cross-compilable:

| Subcommand          | OS gate           | Purpose                                      |
| ------------------- | ----------------- | -------------------------------------------- |
| `serve`             | all               | foreground HTTP server (dev + service entry) |
| `version`           | all               | print build metadata                         |
| `tray`              | `#[cfg(windows)]` | system tray + auto-spawn `serve` in-process  |
| `install-service`   | `#[cfg(windows)]` | register Windows service via SCM             |
| `uninstall-service` | `#[cfg(windows)]` | unregister                                   |

A single binary keeps build / packaging simple and ensures the tray and
service entry points share `serve` semantics by direct function call
rather than process exec. Windows-only crates are declared with
`[target.'cfg(windows)'.dependencies]` so non-Windows builds do not
depend on them.

The Windows service entry point hands off to the same `serve`
function used by the foreground subcommand; the only difference is the
SCM lifecycle wrapper.

## Consequences

Positive:

- One binary, one cargo build target.
- Cross-platform CI runs on Linux without conditional artifact
  handling.
- Tray and service share `serve` initialization, eliminating drift
  between deployment modes.
- Adding future subcommands (e.g. `tepra healthcheck`) follows the
  same pattern.

Negative:

- `tray` and `install-service` are silently absent on Linux `--help`
  output, which may confuse operators. Mitigation: document the gate
  in `--help` epilog.
- The Windows-only crates still appear in `Cargo.lock`. Mitigation:
  optional dependencies behind a `windows` feature flag if lockfile
  noise becomes a problem.

## Alternatives Considered

- **Separate binaries** (`tepra-server`, `tepra-tray`) —
  rejected. Doubles the build / packaging surface and forces IPC or
  re-exec between tray and server for what is logically one process.
- **External wrapper (NSSM)** — rejected. Adds a third-party
  dependency for service registration and obscures lifecycle behavior
  inside the wrapper rather than the binary's own code.
- **Defer Windows deployment to MVP3** — rejected. The target
  environment is Windows; deferring leaves no production story for
  MVP2.

## History

- 2026-06-28: initial version
