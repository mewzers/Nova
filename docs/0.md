# Project Overview

Oxide is a modular application written in Rust using egui/eframe.

## Features

- GUI application
- Modular architecture
- CPU and memory system
- Input handling (keyboard, gamepad)
- Audio support
- Debug tools
- Internationalization (i18n)

## Architecture Philosophy

The project is structured around a central application state (`Oxide`)
and multiple independent modules (CPU, memory, UI, etc.).

Each module is responsible for a specific part of the system.
