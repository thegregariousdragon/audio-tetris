# Audio Tetris

A fully accessible, screen-reader-first arcade Tetris experience built entirely in Rust.

Audio Tetris strips away graphical user interfaces in favor of a native, hyper-responsive audio environment. Leveraging the Tolk screen reader abstraction library for direct speech output, pitch-mapped elevation, and precision stereo panning, the game is designed from the ground up for visually impaired players and audio arcade enthusiasts.

## Features

- **Screen-Reader First & Tolk Integration:** Native integration with Windows screen readers (NVDA, JAWS, Narrator, System SAPI) via Tolk. Includes custom interactive document reading modes for instructions and menus.
- **Multilingual Localization (i18n & l10n):** Native out-of-the-box support for 12 languages and regional dialects with automatic system locale detection on startup and live runtime switching in Settings:
  - English (US & UK)
  - Spanish (Castilian & Latin American)
  - French (France & Canada)
  - Italian
  - German
  - Chinese (Simplified & Traditional)
  - Japanese
  - Korean
- **Interactive 9-Lesson Audio Tutorial:** Hands-on audio lessons teaching lateral movement, stereo panning, pitch elevation, rotations, piece inspection, hold queue swapping, single & 4-line Tetris clears, 10-tone radar sweeps, Zone Mode, and power-up items.
- **Interactive Keyboard Help Mode (Key Describer):** Press **H** on the Main Menu to enter an interactive key exploration mode that speaks the exact function of any key pressed without triggering gameplay actions.
- **Arcade State Machine & Custom Audio UI:** A lightning-fast, custom menu interface with dedicated navigational audio cues (no clunky native Windows toolbars).
- **Dynamic Audio Cues:** Pitch-mapped movement (higher pitch = top of board, lower pitch = bottom) and distinct sound effects for piece rotations, drops, holds, and alignments.
- **Advanced Mechanics:** Utilize the Hold slot, activate the Zone Mode to freeze time, use Power-ups (The Magnet, The Laser, The Nuke), and perform T-Spins, Combos, and Back-to-Back bonuses.
- **Speech Verbosity Customization:** Tailor the voice experience in Settings with customizable piece callouts (Terse vs. Descriptive), scoring details (Simple vs. Advanced), and Zone Mode alerts.
- **Persistent Save/Load & High Scores:** 5 save slots with SQLite database persistence and lifetime statistics tracking across sessions.
- **Background Music Engine:** Real-time multi-track audio engine with ID3 metadata parsing via Lofty.
- **In-App Auto-Updater:** Background check and dedicated update screen communicating with GitHub Releases with safe one-click installation and backup.

## Controls & Keyboard Help Mode

Audio Tetris features dual ergonomic keyboard clusters designed for both one-handed (left or right hand) and traditional two-handed players.

Rather than memorizing a static list of keys, you can explore all keybindings interactively at any time:
1. **Keyboard Help Mode (Key Describer):** Press **H** from the Main Menu. Press any key on your keyboard to hear its assigned function spoken aloud. Press **Escape** twice to exit.
2. **How to Play Screen:** Select **How to Play** from the Main Menu or Pause Menu to read the step-by-step game rules and audio orientation guide line-by-line using your arrow keys.

## Localization (i18n) & Translation Guide

Audio Tetris uses compile-time embedded translation catalogs powered by `rust-i18n`. All user-facing strings are organized into structured JSON dictionaries located in the `locales/` directory.

### Supported Locales

| Locale Code | Language / Dialect | Catalog File |
| :--- | :--- | :--- |
| `en-US` | English (United States) | `locales/en-US.json` |
| `en-GB` | English (United Kingdom) | `locales/en-GB.json` |
| `es-ES` | Spanish (Castilian / Spain) | `locales/es-ES.json` |
| `es-LA` | Spanish (Latin America) | `locales/es-LA.json` |
| `fr-FR` | French (France) | `locales/fr-FR.json` |
| `fr-CA` | French (Canada / Quebec) | `locales/fr-CA.json` |
| `it-IT` | Italian (Italy) | `locales/it-IT.json` |
| `de-DE` | German (Germany) | `locales/de-DE.json` |
| `zh-CN` | Chinese (Simplified / Mandarin) | `locales/zh-CN.json` |
| `zh-TW` | Chinese (Traditional / Mandarin) | `locales/zh-TW.json` |
| `ja-JP` | Japanese (Japan) | `locales/ja-JP.json` |
| `ko-KR` | Korean (Korea) | `locales/ko-KR.json` |

### Catalog Namespaces

Strings are organized into domain-specific namespaces:
- `common`: Generic interface labels (Back, Quit, Confirm, Cancel, Loading, Item Counter).
- `main_menu` / `pause_menu`: Menu options, headers, and spoken index counters.
- `settings`: Setting labels, volume sliders, difficulty toggles, and speech verbosity submenus.
- `in_game`: Piece movement, rotation angles, hold queue, drops, line clears, combos, T-Spins, Zone events, and Game Over.
- `pieces`: Technical names ("Bar", "Square", "T", etc.) and descriptive names ("Long bar", "Left L-shape", etc.).
- `items`: Power-up names (The Magnet, The Laser, The Nuke).
- `key_describer`: Spoken descriptions for every key in Keyboard Help Mode.
- `tutorial`: Lesson titles, step objectives, controls, status indicators, and spoken instructions for all 9 lessons.
- `how_to_play` / `about`: Instructional guides and credits lines.
- `confirm_dialog`: Confirmation modal prompts (New Game, Abandon, Quit, Update).
- `save_load`: Slot descriptions, empty slot announcements, load/save results.
- `leaderboard`: Rank displays, high scores, and lifetime statistics.
- `updater`: Update check statuses, release notes header, and download announcements.

### Variable Interpolation

Dynamic values use the `%{variable}` syntax in JSON:
```json
"game_over": "Game Over! Final Score: %{score}"
```

In Rust, translations are accessed via the `t!` macro:
```rust
let announcement = t!("in_game.game_over", score = 15000);
```

### Adding or Updating Translations

1. **Modify the Master Catalog:** Update or add the new key in `locales/en-US.json`.
2. **Update Target Catalogs:** Add the matching key and translation to all other `locales/*.json` files.
3. **Verify Key Parity:** Run the automated parity test suite to ensure no keys are missing or misspelled:
   ```bash
   cargo test i18n::tests::test_locale_files_exist_and_match_keys
   ```

## Building from Source

Audio Tetris is a native Windows desktop application and requires a specific toolchain to compile successfully.

### Build Requirements

1. **Rust Toolchain:** Latest stable Rust (Rust 2024 Edition, MSRV 1.85+) installed via rustup.
2. **Visual Studio Build Tools:** Ensure the *Desktop development with C++* workload is installed.
3. **LLVM:** Required for the `rust-lld` linker.
4. **CMake and Ninja:** Required for building native C and C++ dependencies seamlessly.
5. **Node.js:** Included for markdown linting to ensure screen reader accessibility of documentation.

### Special Build Flags and Configuration

- **Fast Linking and Caching:** The `.cargo/config.toml` is pre-configured to use `rust-lld.exe` for significantly faster linking, and leverages `sccache.exe` to cache build artifacts and limit build threads to preserve CPU resources for screen readers.
- **Windows Manifest & DPI Awareness:** The `build.rs` script embeds a Windows manifest that enforces Common Controls version 6 and PerMonitorV2 DPI awareness. While this specific game bypasses native MSAA controls in favor of zero-latency raw inputs and direct Tolk speech, maintaining this manifest remains a strict baseline requirement across our `wxDragon` applications to ensure consistent OS-level DPI scaling and window rendering.

### Build Commands

Once your toolchain is configured, you can build and run the game using Cargo:

```bash
cargo build --release
cargo run --release
```

To run all automated test suites and linting checks:

```bash
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

## Attribution and Licenses

**Audio Tetris** was created by **Gregory Lopez** and **Google Antigravity**.

This project relies on the following open source components:

- **Rust Language:** Developed by the Rust Foundation (MIT and Apache 2.0 License).
- **wxDragon:** Native GUI bindings authored by Allen Dang (MIT License).
- **Rodio:** Audio playback and synthesis library authored by Pierre Krieger (Tomaka) and the RustAudio team (MIT and Apache 2.0 License).
- **Tolk:** Screen reader abstraction library authored by Leonard de Ruijter (LGPL License; Rust bindings by Davy Kager and tolk-rs community).
- **Rusqlite / SQLite:** Embedded SQLite database engine authored by John Gallagher, Rusqlite contributors, and D. Richard Hipp (MIT License and Public Domain).
- **rust-i18n:** Compile-time embedded translation catalog framework authored by LongAster and rust-i18n contributors (MIT and Apache 2.0 License).
- **sys-locale:** Cross-platform system locale detection library authored by 1Password and sys-locale contributors (MIT and Apache 2.0 License).
- **Serde & Serde JSON:** High-performance data serialization framework authored by David Tolnay (MIT and Apache 2.0 License).
- **Lofty:** Audio metadata and ID3 tagging library authored by EpocDotFr (MIT and Apache 2.0 License).
- **Reqwest:** Async HTTP client library authored by Sean McArthur and the Reqwest contributors (MIT and Apache 2.0 License).
- **SemVer:** Semantic versioning parser authored by David Tolnay and the SemVer contributors (MIT and Apache 2.0 License).
- **Rand:** Random number generation library authored by The Rand Project Developers (MIT and Apache 2.0 License).
- **Lazy Static:** Macro for static variable initialization authored by Marvin Löbel and the Rust community (MIT and Apache 2.0 License).
- **WinRes:** Windows resource compiler authored by mxre (MIT License).

## License

This project is licensed under the MIT License.  
Copyright (c) 2026 Gregory Lopez and Google Antigravity.
