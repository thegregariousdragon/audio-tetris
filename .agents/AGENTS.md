<RULE[project_branching_model]>
# Branching Model and Protected Main Branch

The `main` branch of this repository is protected and strictly requires pull request reviews. You CANNOT push directly to `main`.

When you are asked to make changes to the codebase, you MUST follow this Git workflow:
1. **Create a Branch**: Create and checkout a new branch with a descriptive name (e.g., `git checkout -b feature/dynamic-music`).
2. **Commit Changes**: Make your changes and commit them to this branch.
3. **Push Branch**: Push the branch to the remote repository (e.g., `git push -u origin feature/dynamic-music`).
4. **Create Pull Request**: Use the GitHub CLI to create a Pull Request against the `main` branch (`gh pr create --title "..." --body "..."`).
5. **Auto-Approve and Merge**: Once the Pull Request is created, use the GitHub CLI to approve it yourself (`gh pr review --approve`) and then merge it (`gh pr merge --squash --delete-branch`). This respects the branch protection while allowing you to complete the workflow automatically.
</RULE[project_branching_model]>

<RULE[accessibility_chat_presentation]>
# Screen Reader & Chat Accessibility Rule

The user uses a screen reader. For accessibility and ease of reading:
- Whenever an implementation plan, research report, architectural proposal, or walkthrough document is generated, you MUST present the full text directly in the chat response.
- Do not rely solely on external file links or separate UI windows for critical documents.
- Keep markdown formatting clean, linear, and accessible (use clear standard headings, explicit bulleted lists, and readable text descriptions).
</RULE[accessibility_chat_presentation]>

<RULE[rust_wxdragon_gui_threading]>
# WxDragon & GUI Threading Rules

- **Main Thread UI Ownership**: wxDragon wraps wxWidgets C++ FFI. All GUI creation, modification, widget destruction, and modal triggers MUST occur strictly on the main GUI thread (`wxdragon::main`).
- **Non-Blocking Event Callbacks**: Event handlers bound to wxDragon elements MUST NEVER perform blocking operations (e.g., heavy file I/O, audio processing, or blocking sleeps). Offload long-running tasks to background worker threads.
- **Cross-Thread GUI Updates**: Background worker threads (such as game logic ticks or audio monitors) MUST NOT touch UI widgets directly. Communicate state updates back to the UI thread using thread-safe channels (`std::sync::mpsc` or `crossbeam`) or atomic/mutex shared state.
</RULE[rust_wxdragon_gui_threading]>

<RULE[audio_and_logic_decoupling]>
# Audio Thread Isolation & Engine Decoupling

- **Dedicated Audio Thread**: Keep sound synthesis and audio playback (`rodio`) on dedicated audio worker thread(s) or sinks to prevent audio stutter/glitches when the UI repaints, resizes, or processes window events.
- **Pure Game Logic Decoupling**: Maintain strict boundaries separating core Tetris grid/tick calculations (`logic.rs`) from presentation (`gui.rs`) and sound generation (`audio.rs`). Logic functions should be pure and easily testable without requiring GUI or sound contexts.
</RULE[audio_and_logic_decoupling]>

<RULE[accessibility_tolk_best_practices]>
# Screen Reader Integration & Speech Fallbacks

- **Non-Blocking Speech**: Screen reader announcements via `tolk` or Windows speech APIs should be lightweight and dispatched without stalling the main UI event loop or game tick loop.
- **Graceful Fallbacks**: Speech output logic must handle scenarios gracefully where no active screen reader (JAWS, NVDA, System SAPI) is loaded on the user's computer without crashing or panicking.
</RULE[accessibility_tolk_best_practices]>

<RULE[rust_code_quality_and_safety]>
# Rust Safety, Error Handling & Code Quality

- **Panic-Free Production Code**: Avoid `.unwrap()` or `.expect()` inside runtime event handlers, audio callbacks, or FFI wrappers. Use explicit `Result` handling, pattern matching, or logged fallbacks to ensure application stability.
- **Linting & Verification**: All code additions must compile cleanly under `cargo check` and adhere to `cargo clippy -- -D warnings`.
</RULE[rust_code_quality_and_safety]>

<RULE[rust_edition_standard]>
# Rust Edition Standard

- **Latest Edition Adoption**: The project must target the latest stable Rust edition. As of the current development cycle, this is the **Rust 2024 Edition**.
- **Migration Protocol**: When upgrading editions, the developer must:
    1. Update `Cargo.toml`.
    2. Run `cargo check` to verify syntax and types.
    3. Run `cargo test` to ensure behavioral consistency.
    4. Verify the application's core functionality (e.g., audio-to-logic timing) via manual testing.
</RULE[rust_edition_standard]>
