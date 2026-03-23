
<h1 align="center">
  <b>Oxide</b>
  <br>
  <sub>A modern CHIP-8 emulator written in Rust</sub>
  <br>
  <a href="https://github.com/OxideChip8/Oxide">
    <picture>
      <source srcset="images/svg/Oxide.svg" type="image/svg+xml">
      <img src="images/png/Oxide.png" width="290">
    </picture>
  </a>
  <br>

  <a href="https://github.com/OxideChip8/Oxide/releases">
    <img src="https://img.shields.io/github/v/release/OxideChip8/Oxide" width="185">
  </a>
  <a href="https://github.com/OxideChip8/Oxide/releases/latest">
    <img src="https://img.shields.io/github/downloads/OxideChip8/Oxide/total?color=45E600" width="160">
  </a>
  <img src="https://img.shields.io/badge/Rust-100%25-red" width="145">
  <br>
  <a href="https://github.com/OxideChip8/Oxide/milestones">
    <img src="https://img.shields.io/badge/Current progress-%25-magenta" width="210">
  </a>
</h1>

### Oxide is a modular CHIP-8 emulator written in Rust with an egui/eframe interface, multilingual support, debugging tools, and configurable input/video/audio settings.
### It is compatible with Windows, macOS, and Linux.


<h1 align="left">🎯 Project Goal :</h1>

This project aims to learn and practice the Rust programming language through a concrete use case: building a CHIP-8 emulator.

It helps to understand key Rust concepts (memory management, structs, ownership, etc.) while also introducing the fundamentals of emulator development, such as the instruction cycle, input/output handling, and graphics rendering.

Beyond simply recreating a basic emulator, the objective is also to design a modern CHIP-8 emulator that includes features and interfaces inspired by contemporary emulators. This may include a user-friendly interface, debugging tools, configurable controls, save states, and performance optimizations.

The goal is to provide a solid foundation for both mastering Rust and understanding how emulators work, making it easier to build similar projects for other systems in the future.
<h2>
  To view the project roadmap :
  <a href="https://github.com/OxideChip8/Oxide/blob/main/ROADMAP.md">ROADMAP.md</a>
</h2>

<h1 align="left"> Compatibility :

<p align="center">
 <a href="https://github.com/OxideChip8/Oxide/releases">
    <picture>
      <source srcset="images/svg/W11.svg" type="image/svg+xml">
      <img src="images/png/W11.png" width="150">
    </picture>
  </a>
  
  <a href="https://github.com/OxideChip8/Oxide/releases">
    <picture>
      <source srcset="images/svg/Linux.svg" type="image/svg+xml">
      <img src="images/png/Linux.png" width="140">
    </picture>
  </a>
  
  <a href="https://github.com/OxideChip8/Oxide/releases">
    <picture>
      <source srcset="images/svg/Tahoe.svg" type="image/svg+xml">
      <img src="images/png/Tahoe.png" width="150">
    </picture>
  </a>
  <br>
  
  <p align="center"><b>Windows 11, Linux and macOS</b></p>
  </p>
</h1>


<h1 align="left">
  Emulator :
  <br><br>
    
  <p align="center">
      <picture>
      <source srcset="images/svg/Window.svg" type="image/svg+xml">
      <img src="images/png/Window.png" width="900">
  </picture>
  </p>
  
<h1 align="left">
  Gallery :
  <br><br>


|                Space Invaders                 |                   Snake                   |
| :------------------------------------------:  |:----------------------------------------: |
| <picture><source srcset="./images/svg/SpaceInvaders.svg" type="image/svg+xml"><img src="./images/png/SpaceInvaders.png"></picture> | <picture><source srcset="./images/svg/Snake.svg" type="image/svg+xml"><img src="./images/png/Snake.png"></picture> 

|                    Tetris                    |                    Pong                    |
| :------------------------------------------: | :----------------------------------------: |
| <picture><source srcset="./images/svg/Tetris.svg" type="image/svg+xml"><img src="./images/png/Tetris.png"></picture> | <picture><source srcset="./images/svg/Pong.svg" type="image/svg+xml"><img src="./images/png/Pong.png"></picture> |


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
3. Run `Oxide` (depending of the OS)
4. Click **Game → Load game** and select a `.ch8, .rom or .bin` file

<h1 align="left">Controls :</h1>

The CHIP-8 uses a 16-key hexadecimal keypad mapped to your keyboard by default :


| 1 | 2 | 3 | C |
|:------:|:---:|:---:|:---:|
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

All bindings are fully configurable in **Settings → Controls**.

<h1 align="left">Built With :</h1>

- [Rust](https://www.rust-lang.org/)
- [egui / eframe](https://github.com/emilk/egui)
- [rodio](https://github.com/RustAudio/rodio)
- [gilrs](https://gitlab.com/gilrs-project/gilrs)
- [rfd](https://github.com/PolyMeilex/rfd)

<h1 align="left">License :</h1>

[MIT](https://github.com/OxideChip8/Oxide?tab=MIT-1-ov-file)
