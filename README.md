
<h1 align="center">
  <b>Oxide - Chip8</b>
  <br>
  <a href="https://github.com/OxideChip8/Oxide"><img src="images/Oxide.png" width="290"></a>
  <br>
</h1>

Oxide is a modular CHIP-8 emulator written in Rust with an egui/eframe interface, multilingual support, debugging tools, and configurable input/video/audio settings.
It is compatible with Windows, macOS, and Linux.
<h1 align="center">
  <a href="https://github.com/OxideChip8/Oxide/releases"><img src="images/W11.png" width="150"></a>
  <a href="https://github.com/OxideChip8/Oxide/releases"><img src="images/Linux.png" width="140"></a>
  <a href="https://github.com/OxideChip8/Oxide/releases"><img src="images/Tahoe.png" width="150"></a>
  <br>
  <b>Windows 11</b>
  <b>Linux</b>
  <b>macOS</b>
  <br>
</h1>

<h1 align="center">
  <a href="https://github.com/OxideChip8/Oxide/releases/latest">
    <img src="https://img.shields.io/github/v/release/OxideChip8/Oxide" width="185">
  </a>
  <a href="https://github.com/OxideChip8/Oxide/releases/latest">
    <img src="https://img.shields.io/github/downloads/OxideChip8/Oxide/total?color=45E600" width="160">
  </a>
  <img src="https://img.shields.io/badge/Rust-100%25-red" width="145">
  <br>
</h1>
<h1 align="left">
  Emulator :
  <br><br>
  

<h1 align="left">
  Gallery :
  <br><br>

|                Space Invaders                 |                   Snake                   |
| :------------------------------------------:  |:----------------------------------------: |
| ![Space Invaders](./images/SpaceInvaders.png) | ![Snake](./images/Snake.png) |


|                    Tetris                    |                    Pong                    |
| :------------------------------------------: | :----------------------------------------: |
| ![Tetris](./images/Tetris.png) | ![Pong](./images/Pong.png) |

<h1 align="left">Features :</h1>

- 🎮 Full CHIP-8 CPU (35 opcodes) with configurable quirks (CHIP-8, CHIP-48, SUPER-CHIP)
- 🖥️ 64×32 pixel display with 1x–5x scaling
- 🔊 Audio buzzer with volume control
- 🕹️ Keyboard, mouse and gamepad support (gilrs)
- 💾 Save/Load states (3 slots per ROM)
- 🌍 12 languages (EN, FR, ES, IT, DE, PT, RU, ZH, JA, KO, AR, HI)
- 🔧 Configurable controls and keyboard shortcuts
- 🖥️ Debug terminal with search, export and live diagnostics
- 🪟 Windows single-instance protection

  <h1 align="left">Getting Started :</h1>

1. Download the latest release for your OS above
2. Extract the zip
3. Run `Oxide.exe for Windows 11, Oxide.dmg for macOS and Oxide for Linux`
4. Click **Game → Load game** and select a `.ch8` ROM file

<h1 align="left">Controls :</h1>

The CHIP-8 uses a 16-key hexadecimal keypad mapped to your keyboard by default :

| CHIP-8 | 1 | 2 | 3 | C |
|:------:|:---:|:---:|:---:|:---:|
| **4** | Q | W | E | R |
| **7** | A | S | D | F |
| **A** | Z | X | C | V |

All bindings are fully configurable in **Settings → Controls**.

<h1 align="left">Built With :</h1>

- [Rust](https://www.rust-lang.org/)
- [egui / eframe](https://github.com/emilk/egui)
- [rodio](https://github.com/RustAudio/rodio)
- [gilrs](https://gitlab.com/gilrs-project/gilrs)
- [rfd](https://github.com/PolyMeilex/rfd)

<h1 align="left">License :</h1>

MIT
