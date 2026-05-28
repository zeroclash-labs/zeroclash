# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

ZeroClash — a pure-Rust GPUI desktop GUI for the mihomo (Clash Meta) proxy core. Rewrites [clash-verge-rev](https://github.com/clash-verge-rev/clash-verge-rev) (Tauri+React) into a single-language Rust codebase. Reference project at `.exclude/clash-verge-rev/` (gitignored, study only).

## Build, test, run

```sh
cargo check --workspace              # fast compile check
cargo test --workspace               # run all tests
cargo clippy --workspace             # strict lints (40+ deny rules)
cargo fmt --all -- --check           # verify formatting
cargo build --release                # release build — build.rs downloads mihomo core
cargo run -p zeroclash               # launch the GUI
cargo run -p zeroclash-cli -- --help # CLI usage
```

Linux builds need `libgtk-3-dev libxdo-dev` + `libayatana-appindicator3-dev` (Ubuntu 24.04+) or `libappindicator3-dev` (< 24.04).

## Architecture

**Workspace**: 8 library crates under `crates/`, 2 app binaries under `apps/`.

`apps/zeroclash` (GUI) → `zeroclash-ui` → `zeroclash-core` → all other crates.
`apps/zeroclash-cli` (CLI) → `zeroclash-core`.

| Crate | Purpose |
|-------|---------|
| `zeroclash-core` | All business logic — config, mihomo client, profiles, core installer, enhance pipeline, connections, backup, service, system proxy. **No GUI dependencies.** |
| `zeroclash-ui` | GPUI desktop GUI — AppState command queue, 6 views, sidebar, design tokens, tray/hotkey managers. |
| `zeroclash-draft` | `Draft<T>` — `Arc<RwLock<(committed, draft)>>` for zero-copy reads with lazy copy-on-write. |
| `zeroclash-i18n` | 13 locales via `rust-i18n`, locale resolution with aliases, `t!` macro. |
| `zeroclash-limiter` | `AtomicU64`-based rate limiter with `SystemClock`. Used for tray click debouncing. |
| `zeroclash-logging` | 20 log type categories, `logging!`/`logging_error!` macros, `init_logger()` for flexi_logger init. |
| `zeroclash-signal` | OS signal handling (SIGTERM/SIGINT on Unix, Ctrl+C on Windows) for graceful shutdown. |
| `zeroclash-sysinfo` | System info (OS/kernel/arch), app info (version/mode/admin), network interfaces. |

### zeroclash-core module map

`config` — VergeConfig + Config (Draft-based) | `mihomo` — MihomoClient (REST) + CoreManager (process) + `resolve_core_path()` | `core_installer` — runtime & build-time mihomo core download from GitHub releases | `paths` — centralized platform directories (cache/config/data/log) via `dirs` crate | `profile` — ProfileStore, PrfItem, subscription import | `enhance/` — merge → script (Boa JS) → seq → chain → TUN → DNS | `connection` — WebSocket-based connection monitoring | `service` — Windows service / Linux systemd | `sys` — AutoStart, SystemProxy, singleton, notifications | `backup` — local zip + WebDAV | `media_unlock` — geo-unlock checker | `constants` — ports, file paths

### zeroclash-ui structure

`state.rs` — `AppState` (central model), `UiCommand` enum, `process_commands()` command queue, `poll_events()` for tray/hotkey | `design.rs` — `Colors` tokens, spacing/radius/font scales | `theme.rs` — Theme global, light/dark auto-detect | `views/` — dashboard, proxies, profiles, connections, logs, settings | `components/` — card, log_viewer, settings_group, traffic_graph | `tray.rs` — system tray with mode/proxy/TUN/quit menu | `hotkey.rs` — macOS global hotkeys

## Gotchas

- Edition **2024** — `let` chains, `unsafe extern` blocks.
- **macOS 26 (Darwin 25)**: GPUI v1.3.7 has an `objc` 0.2.7 incompatibility. Run once after `cargo fetch`:
  ```sh
  python3 scripts/patch-gpui.py
  ```
- Clippy deny lints include `panic`, `unimplemented`, `unused_async`, `future_not_send`, `await_holding_lock`, `cognitive_complexity`. The UI crate allows `large_types_passed_by_value` and `needless_pass_by_ref_mut` (GPUI architecture requirement).
- Async bridging: use `pollster::block_on()` in GPUI's sync `process_commands()`; spawn long-running work via `crate::runtime::handle().spawn()`.
- `build.rs` downloads the mihomo core binary during `--release` builds and places it in the output directory. Skip in debug builds for fast iteration.
- `.exclude/` contains clash-verge-rev source (manually placed, not a submodule).
- License: CC-BY-NC-SA-4.0 (non-commercial).

## Commit style

Conventional commits (`feat:`, `fix:`, `refactor:`). Push directly to `main`.

## Existing automation

Skills: `/fix-issue` (GitHub issue → PR), `/gpui-design` (shadcn/ui-style GPUI components), `/new-gpui-view` (scaffold a GPUI view), `/verify` (check + test + clippy + fmt).
Hook: rustfmt auto-runs on every `Write`/`Edit` to `*.rs` files.
