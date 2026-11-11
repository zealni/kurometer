# Implementation Progress

The application uses Rust and native Win32. The taskbar meter is hosted under `Shell_TrayWnd`; a separate controller owns settings, details, timers, shell notifications and the notification-area icon. UI Automation placement scans run off the UI thread. If Explorer temporarily refuses parenting, Kurmon retries and can fall back to an overlay.

The repository includes live CPU, RAM, GPU and network sampling; per-user settings persistence; a worker thread; taskbar placement; settings and overview panels; and a per-user installer. See `docs/verification.md` for verified behavior and remaining qualification work.
