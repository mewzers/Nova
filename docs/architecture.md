# Oxide Architecture

## Overview

```mermaid
flowchart TD

    %% Entry point
    Main[main.rs] --> App[app.rs]

    %% App structure
    App --> Init[Initialize app]
    App --> Loop[Main loop]

    %% Core systems
    Loop --> CPU[cpu.rs]
    Loop --> Memory[memory.rs]
    Loop --> Display[display.rs]
    Loop --> Input[gamepad.rs / keypad.rs]

    %% Additional systems
    App --> Audio[audio.rs]
    App --> UI[ui/]
    App --> I18n[i18n.rs + json]

    %% Feedback loop
    CPU --> Memory
    Memory --> Display
    Display --> Input
    Input --> Loop
