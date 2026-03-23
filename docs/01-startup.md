# Application Startup

This document describes how the application initializes and starts.

## Startup Flow

```mermaid
flowchart TD

    A[main] --> B[Single instance check]

    B -->|OK| C[starting_Oxide]
    B -->|Already running| X[Exit]

    C --> D[Build viewport]
    D --> E[Load window icon]

    E --> F[Create NativeOptions]

    F --> G[Read CLI args]

    G --> H[run_native]

    H --> I[Setup fonts]

    I --> J[Load or create Oxide]

    J --> K[Reset runtime state]
    K --> L[Apply CLI file]

    L --> M[terminal_ready]

    M --> N[Start GUI loop]

    N --> O[oxide_exited]
```
## Key Points

- The application starts from `main`.
- A single-instance check is performed on Windows.
- A splash screen viewport is created before launching the app.
- The GUI is powered by `eframe`.
- The main application state is handled by the `Oxide` struct.
- Runtime state is reset on each startup.

---
