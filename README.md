# Audio Tetris

A fully accessible, screen-reader-first arcade Tetris experience built entirely in Rust.

Audio Tetris strips away graphical user interfaces in favor of a native, hyper-responsive audio environment. Leveraging Microsoft Active Accessibility (MSAA), advanced gamepad integrations, and meticulously engineered keyboard clusters, the game is designed from the ground up for visually impaired and sighted players alike.

## Features

- **Screen-Reader First:** Native integration with Windows screen readers via `Tolk`. Includes custom interactive document reading modes for instructions and menus.
- **Arcade State Machine:** A lightning-fast, custom menu interface with dedicated navigational audio cues (no clunky native Windows toolbars).
- **Advanced Gamepad Support:** Play with any XInput compatible controller (Xbox, Luna, Generic) using the 60Hz polled `gilrs` backend.
- **Dynamic Audio Cues:** Pitch-mapped movement (higher pitch = top of board, lower pitch = bottom) and distinct SFX for piece rotations, drops, holds, and alignments.
- **The "Hold" Mechanic:** Swap your actively falling piece into a "Hold" slot once per drop.
- **Ergonomic Playstyles:** Dedicated, mirrored keyboard clusters ensuring comfortable play for one-handed (left or right) and traditional two-handed players.

## Controls

### Gamepad (XInput)
- **Move:** D-Pad Left / Right
- **Soft Drop:** D-Pad Down
- **Hold Piece:** D-Pad Up
- **Hard Drop:** Right Trigger (RT)
- **Radar Sweep:** Left Trigger (LT)
- **Rotate Left / Right:** Left Bumper (LB) / Right Bumper (RB)
- **Background Music:** Click Left/Right Analog Sticks (L3/R3)
- **Quick Settings / Pause:** Start Button
- **Select / Back (Menus):** A Button / B Button

### Keyboard Playstyles
Audio Tetris natively supports three distinct keyboard playstyles. *Note: Arrow Keys are disabled during gameplay to encourage ergonomic posture, but remain active for menu navigation.*

**1. Left-Handed One-Hand**
- Move: `A`, `S` (Drop), `D`
- Rotate/Hold: `Z`, `X`, `C` (Hold)
- Music: `Q` (Prev), `W` (Mute), `E` (Next)

**2. Right-Handed One-Hand**
- Move: `L`, `;` (Drop), `'` (Apostrophe)
- Rotate/Hold: `,` (Comma), `.` (Period), `/` (Hold)
- Music: `P` (Prev), `[` (Mute), `]` (Next)

**3. Traditional Two-Handed**
- Move: Right hand on `L`, `;`, `'`
- Rotate/Hold: Left hand on `Z`, `X`, `C`
- *(Spacebar to Hard Drop, R for Radar, Tab to Pause)*

## Building from Source

Ensure you have Rust and Cargo installed, then run:

```bash
cargo build --release
cargo run --release
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
Copyright (c) 2026 Gregory Lopez.
