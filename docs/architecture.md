# Oxide Architecture

## Overview

```mermaid
flowchart TD
    Main[main.rs] --> App[app.rs]

    App --> CPU[cpu.rs]
    App --> Memory[memory.rs]
    App --> Display[display.rs]
    App --> Input[gamepad.rs / keypad.rs]
    App --> Audio[audio.rs]
    App --> UI[ui/]
    App --> I18n[i18n.rs + json]
