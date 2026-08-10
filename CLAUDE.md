# Audio Tetris - Project Guidelines & AI Agent Rules

## Project Branching & Release Workflow
- **Protected `main` Branch**: Never push directly to `main`. All changes MUST go through Pull Requests.
- **Git Feature Branch Workflow**:
  1. Create a feature branch: `git checkout -b feature/<descriptive-name>`
  2. Commit changes to the feature branch.
  3. Push branch: `git push -u origin feature/<descriptive-name>`
  4. Create PR via GitHub CLI: `gh pr create --fill`
  5. Enable auto-merge: `gh pr merge --squash --auto --delete-branch`
- **Release Protocol**: Releases are triggered by pushing a signed Git tag starting with `v` (e.g. `v0.1.0`):
  ```bash
  git tag vX.Y.Z
  git push origin vX.Y.Z
  ```

## Pre-Flight Quality Checks
Before creating any Pull Request, developers and AI agents MUST run the following pre-flight checks locally:
1. `cargo fmt --check`
2. `cargo check --all-targets --all-features`
3. `cargo test --all-targets --all-features`
4. `cargo clippy --all-targets --all-features -- -D warnings`

## Rust 2024 Edition & MSRV Standard
- **Target Edition**: Rust 2024 Edition (`edition = "2024"` in `Cargo.toml`).
- **MSRV**: Minimum Supported Rust Version is `rust-version = "1.85"`.

## Architecture & Code Safety Rules
- **WxDragon GUI Threading**: All wxWidgets/wxDragon UI creation, modification, and modal triggers MUST occur on the main thread (`wxdragon::main`). Offload background work to worker threads using `std::sync::mpsc` channels.
- **Audio Thread Isolation**: Keep sound synthesis (`rodio`) on dedicated audio worker threads to prevent stuttering during UI repaints.
- **Pure Game Logic Decoupling**: Keep core Tetris grid/tick calculations (`logic.rs`) pure and decoupled from GUI (`gui.rs`) and Audio (`audio.rs`).
- **Panic-Free Production Code**: Avoid `.unwrap()` or `.expect()` inside runtime handlers or audio callbacks. Use explicit `Result` handling.
- **Screen Reader Speech (`tolk`)**: Screen reader announcements via `tolk` or SAPI must be non-blocking and fail gracefully if no screen reader is loaded.

## Screen Reader Accessibility & Presentation Protocol
- **Screen Reader Formatting**: The user uses a screen reader. Present all implementation plans, walkthroughs, research reports, and responses directly in the chat output in clean, linear markdown.
- **Empirical Verification Protocol**: Never claim a bug is fixed or feature is complete without running local `cargo test` / `cargo check` build steps to verify results empirically.
