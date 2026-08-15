# Audio Tetris

A fully accessible, screen-reader-first arcade Tetris experience built entirely in Rust.

Audio Tetris strips away graphical user interfaces in favor of a native, hyper-responsive audio environment. Leveraging the Tolk screen reader abstraction library for direct speech output and meticulously engineered keyboard clusters, the game is designed from the ground up for visually impaired players.

## Features

- **Screen-Reader First:** Native integration with Windows screen readers via Tolk. Includes custom interactive document reading modes for instructions and menus.
- **Arcade State Machine:** A lightning-fast, custom menu interface with dedicated navigational audio cues (no clunky native Windows toolbars).
- **Keyboard Help Mode:** Press H on the Main Menu to enter an interactive mode that speaks the function of every key.
- **Dynamic Audio Cues:** Pitch-mapped movement (higher pitch = top of board, lower pitch = bottom) and distinct sound effects for piece rotations, drops, holds, and alignments.
- **Advanced Mechanics:** Utilize the Hold slot, activate the Zone Mode to freeze time, use Power-ups, and perform T-Spins and Combos.
- **Ergonomic Playstyles:** Dedicated keyboard clusters ensuring comfortable play for one-handed (left or right) and traditional two-handed players.

## Controls

Audio Tetris features keyboard clusters so you can play your way!

- **Left Cluster:** Move with W, A, S, D. Rotate with Z or X. Hold with C. Inspect piece with V. Zone with Q. Radar with E. Item with Left Shift.
- **Right Cluster (4x3 Grid):** Move with Arrow Keys. Rotate with Comma or Period. Hold with Slash (lower right anchor). Inspect piece with Semicolon. Zone with K. Radar with L. Item with Right Shift.
- **Music Controls:** Previous track with I. Mute/Unmute with O. Next track with P.
- **Global Actions:** Hard Drop with W, Up Arrow, or Spacebar.

## Building from Source

Audio Tetris is a native Windows desktop application and requires a specific toolchain to compile successfully.

### Build Requirements

1. **Rust Toolchain:** Install via rustup.
2. **Visual Studio Build Tools 2026:** Ensure the Desktop development with C++ workload is installed.
3. **LLVM:** Required for the rust-lld linker.
4. **CMake and Ninja:** Required for building native C and C++ dependencies seamlessly.
5. **Node.js:** Included for markdown linting to ensure perfect screen reader accessibility of documentation.

### Special Build Flags and Configuration

- **Fast Linking and Caching:** The `.cargo/config.toml` is pre-configured to use `rust-lld.exe` for significantly faster linking, and leverages `sccache.exe` to cache build artifacts and limit build threads to preserve CPU resources for screen readers.
- **Windows Manifest & DPI Awareness:** The `build.rs` script embeds a Windows manifest that enforces Common Controls version 6 and PerMonitorV2 DPI awareness. While this specific game bypasses native MSAA controls in favor of zero-latency raw inputs and direct Tolk speech, maintaining this manifest remains a strict baseline requirement across our `wxDragon` applications to ensure consistent OS-level DPI scaling and window rendering.

### Build Commands

Once your toolchain is configured, you can build and run the game using Cargo:

```bash
cargo build --release
cargo run --release
```

## Attribution and Licenses

**Audio Tetris** was created by **Gregory Lopez** and **Google Antigravity**.

This project relies on the following open source components:

- **Rust Language:** Developed by the Rust Foundation (MIT and Apache 2.0 License).
- **wxDragon:** Native GUI bindings authored by Allen Dang (MIT License).
- **Rodio:** Audio playback and synthesis library authored by Pierre Krieger (Tomaka) and the RustAudio team (MIT and Apache 2.0 License).
- **Tolk:** Screen reader abstraction library authored by Leonard de Ruijter (LGPL License; Rust bindings by Davy Kager and tolk-rs community).
- **Rusqlite / SQLite:** Embedded SQLite database engine authored by John Gallagher, Rusqlite contributors, and D. Richard Hipp (MIT License and Public Domain).
- **Serde & Serde JSON:** High-performance data serialization framework authored by David Tolnay (MIT and Apache 2.0 License).
- **Lofty:** Audio metadata and ID3 tagging library authored by EpocDotFr (MIT and Apache 2.0 License).
- **Rand:** Random number generation library authored by The Rand Project Developers (MIT and Apache 2.0 License).
- **Lazy Static:** Macro for static variable initialization authored by Marvin Löbel and the Rust community (MIT and Apache 2.0 License).
- **WinRes:** Windows resource compiler authored by mxre (MIT License).

## License

This project is licensed under the MIT License.
Copyright (c) 2026 Gregory Lopez and Google Antigravity.

