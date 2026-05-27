# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

ZeroClash — a pure-Rust GPUI desktop GUI for the mihomo (Clash Meta) proxy core. Rewrites [clash-verge-rev](https://github.com/clash-verge-rev/clash-verge-rev) (Tauri+React) into a single-language Rust codebase. The reference project lives in `.exclude/clash-verge-rev/` (gitignored, for study only). Consult it when implementing features that correspond to existing clash-verge-rev behavior.

## Build, test, run

```sh
cargo check --workspace              # fast compile check
cargo test --workspace               # run all tests
cargo clippy --workspace             # strict lints
cargo fmt --all -- --check           # verify formatting
cargo build --release                # release build (apps only)
cargo run -p zeroclash               # launch the GUI
cargo run -p zeroclash-cli -- --help # CLI usage
```

CI runs `check`/`test`/`clippy`/`fmt` on push/PR via `.github/workflows/ci.yml`. Linux CI installs `libgtk-3-dev libxdo-dev` plus `libayatana-appindicator3-dev` (fallback to `libappindicator3-dev` on older Ubuntu).

On macOS 26+, run before building:
```sh
python3 scripts/patch-gpui.py  # one-time GPUI compatibility fix
```

## Architecture

All business logic lives in `zeroclash-core` — a platform-agnostic crate with **no GUI dependencies**. The GPUI app (`crates/zeroclash-ui`) and CLI (`apps/zeroclash-cli`) are thin wrappers.

- **Config system**: `zeroclash-draft` provides `Draft<T>` — `Arc<RwLock<(committed, optional draft)>>` for zero-copy reads with lazy copy-on-write.
- **Enhance pipeline**: `zeroclash-core::enhance` processes clash configs through merge → script (Boa JS) → seq → chain → TUN → DNS.
- **GUI**: GPUI v1.3.7 (Zed's GPU-accelerated framework). Design tokens in `crates/zeroclash-ui/src/design.rs`. Views in `crates/zeroclash-ui/src/views/`.
- **CLI**: clap-derived subcommands for core/config/profile/log operations.

## Gotchas

- Edition **2024** — Rust edition-specific syntax expected (`let` chains, `unsafe extern` blocks).
- **macOS 26 (Darwin 25)**: GPUI v1.3.7 is incompatible due to `objc` 0.2.7 `ClassDecl` not working. Run `python3 scripts/patch-gpui.py` once after `cargo fetch` to apply the workaround.
- Linux builds need system deps: `libgtk-3-dev libxdo-dev` plus `libayatana-appindicator3-dev` (Ubuntu 24.04+) or `libappindicator3-dev` (Ubuntu < 24.04).
- Clippy `deny` lints include: `unimplemented`, `panic`, `unused_async`, `future_not_send`, `await_holding_lock`.
- `.exclude/` contains the reference clash-verge-rev source (not a submodule, manually placed).
- License is CC-BY-NC-SA-4.0 (non-commercial).
