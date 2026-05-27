---
name: verify
description: Run the full verification suite — check, test, clippy, and format check. Use after making changes to ensure nothing is broken.
---

Run all verification steps in order, stopping on first failure:

```sh
cargo check --workspace && cargo test --workspace && cargo clippy --workspace && cargo fmt --all -- --check
```

If `clippy` fails, fix the warnings before continuing. If `fmt` fails, run `cargo fmt --all` and re-check.

On macOS, also verify the GPUI patch is applied:
```sh
grep -q 'objc_getAssociatedObject' ~/.cargo/git/checkouts/zed-*/f1567cf/crates/gpui_macos/src/platform.rs || echo "WARNING: GPUI patch not applied — run python3 scripts/patch-gpui.py"
```
