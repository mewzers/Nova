# Oxide — User Interface & Settings

Documentation of the graphical user interface, windows, and parameter flow.

---

## Main Window Layout

```mermaid
block-beta
    columns 1
    TopBar["Menu Bar (30 px)\nGame | Emulator | Video | Controls | Shortcuts | Debug | Theme ☀/🌙/🌟"]
Display["Central panel\nCHIP-8 display (64×32 × scale)\nPause/message overlay"]
BottomBar["Status Bar (30 px)\nVersion | ROM | Status | CPU Hz | FPS | Volume"]
```

---

## Secondary windows (detached viewports)

```mermaid
graph LR
Main["Main Window\n(root viewport)"]


Settings["Settings\n(immediate viewport)\ncentered on the main window"]
Terminal["Debug console\n(immediate viewport)\nto the right of the main window"]


Main -->|"window_settings = true"| Settings
Main -->|"terminal_active = true"| Terminal
Settings -->|"OK / Cancel / ✕"| Main
Terminal -->|"✕ or checkbox"| Main
```

---

## Settings tabs

```mermaid
flowchart LR
subgraph Tabs["Tabs"]
T1["Emulator\nTheme, Language,\nCPU Hz"]
T2["Video\nVSync, Scale"]
T3["Audio, Volume"]
T4["Controls\n16-key mapping\nCHIP-8"]
T5["Shortcuts\nMapping 11 keyboard shortcuts"]
T6["Debug\nTerminal, Quirks\nCHIP-8"]
end


subgraph Actions["Footer buttons"]
B1["OK → apply + close"]
B2["Apply → apply"]
B3["Default → Reset tab"]
B4["Cancel → Restore Snapshot"]
end
```

---

## Temporary data flow (settings)

```mermaid
stateDiagram-v2
[*] --> Closed


Closed --> Open: Open\n(snapshot of live values)
Open --> Open: Modification\n(temp_* updated)
Open --> Live: Apply / OK\napply_temp_values()
Open --> Live: Cancel\nrestore_snapshots()
Live --> Closed: Window closed
Live --> [*] : save() au quit


note right of Open
temp_theme, temp_langue,
vsync_time, video_scale_time
temp_touches, temp_raccourcis,
temp_quirks, temp_son_active...
end note

```

---

## Key Bindings (Controls)

```mermaid
sequenceDiagram
    actor User
    participant UI as Controls Tab
    participant App

    User->>UI: Click on a CHIP-8 button
    UI->>App: binding_key = Some(index)\nbinding_key_started = Instant::now()\nbinding_key_skip_first_click = true
    UI->>UI: Displays "..." on the button

    alt Timeout 3 seconds
        UI->>App: binding_key = None
    else Keyboard key detected
        UI->>App: temp_touches[index] = label
        UI->>App: binding_key = None
    else Mouse click detected (skip first)
        UI->>App: temp_touches[index] = "MouseLeft/Right/..."
        UI->>App: binding_key = None
    end
```

---

## Keyboard Input Pipeline → CHIP-8

```mermaid
flowchart TD
    KB["Keyboard (egui InputState)\nkeys configured in temp_keys"]
    Mouse["Mouse (egui PointerButton)\nbuttons configured in temp_touches"]
    GP["Gamepad (gilrs)\npoll_chip8_keys()"]
    Term["Debug terminal\nterminal_keypad_states[16]"]

    OR["Logical OR\nstates[i] = keyboard || mouse || gamepad || terminal"]

    Keypad["Keypad.set_all(states)\n→ CPU.cycle() reads is_pressed()"]

    KB --> OR
    Mouse --> OR
    GP --> OR
    Term --> OR
    OR --> Keypad
```

---

## Available themes

```mermaid
flowchart LR
    K["🌟 Kiwano\n(défaut)\nRouge bordeaux\négui custom visuals"]
    D["🌙 Dark\négui::Visuals::dark()"]
    L["☀ Light\négui::Visuals::light()"]

    K -->|"clic icône"| D
    D -->|"clic icône"| L
    L -->|"clic icône"| K
```

---

## Debug Console — Features

```mermaid
graph TD
    subgraph Terminal["Oxide Console (separate viewport)"]
        Logs["Log area\n(multiline TextEdit\nread-only)"]
        Search["Search bar\n(line filter)"]
        BtnReport["Test Report button\nemit_test_report()"]
        BtnExport["Export Logs button\nrfd::FileDialog → .txt"]
    end

    subgraph Content["Log Content"]
        Boot["Boot logs\n(seed_terminal_boot_logs)"]
        Config["Config changes\n(log_config_changes)"]
        Status["Status messages\n(update_terminal_log)"]
        Report["F9 test report\nPC, logs, quirks..."]
    end

    BtnReport --> Report
    Boot --> Logs
    Config --> Logs
    Status --> Logs
    Report --> Logs
    Search --> |"filter"| Logs
    BtnExport --> |"write file"| Disk[("Disk .txt")]

    subgraph Files["Auto log files"]
        AppLog["logs/app/latest.logs\n→ archived as .zip on startup"]
        EmuLog["logs/emulator/latest.logs\n→ archived as .zip on startup"]
    end

    Logs --> AppLog
    Status --> EmuLog
```
