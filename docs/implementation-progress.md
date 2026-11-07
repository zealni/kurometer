# Implementation progress

Plan: docs/superpowers/plans/2026-11-05-taskbar-monitor.md

## Decisions

- Use Rust and Win32 native UI instead of C#/WPF. Rust 1.99.1 MSVC is installed and compiled the live probe successfully. No .NET installation is needed.
- User explicitly approved temporary taskbar alignment/auto-hide changes and Explorer restart for verification. Center, auto-hide Off, and grouping Always were restored after experiments.
- Live expanded taskbar left only 71 pixels; the full original compatibility gate failed. User explicitly chose limited taskbar compatibility: centered/grouped with enough free space; hide the meter in crowded configurations and retain tray access. A separate AppBar was offered and not chosen.

## Hosting revision (2026-11-07)

- The meter is now a child of `Shell_TrayWnd`, following TrafficMonitor's approach; a separate hidden top-level controller owns the tray icon, panels, timers and shell messages so none of them enter Explorer's window tree.
- UI Automation geometry moved to a windowless MTA worker, and the meter's IME context is removed before parenting.
- Explorer can refuse `SetParent` (`ERROR_INVALID_PARAMETER` observed while the shell was busy); the app keeps the previous topmost overlay as fallback and retries at most six times at 5-second intervals, resetting on shell events.
- Verified embedded in the taskbar accessibility tree, coexistence with Explorer's taskbar menu and Start menu, and automatic reattachment after an Explorer restart.
- Hosted mode cannot start a caption move, so drag/resize is disabled; width and position stay in Settings.

## Evidence

- Native probe rendered in taskbar and opened its sample-data disclosure.
- Actual button rectangles obtained using Windows UI Automation; raw child rectangles were unreliable.
- Four placement tests: relevant positive cases failed before implementation; final `cargo test` passed 4/4.
- Probe survived Explorer restart on actual desktop.
- Auto-hide defect reproduced; fixed probe hid during auto-hide and restored to (8,1020), verified by desktop and completed process output.

## Current work

Implement real native metrics, followed by tray/details/settings and packaging. The disposable probe is not the application and its values are sample data.

## Revised implementation map

- Cargo.toml / Cargo.lock: native Windows dependency and binaries.
- src/main.rs: native application lifecycle, taskbar meter, tray, detail/settings controls.
- src/metrics.rs: CPU/RAM/network/GPU sampling and interval arithmetic.
- src/settings.rs: validated local settings and atomic persistence.
- tools/placement.rs: tested interval geometry, reused by application.
- tools/taskbar_probe.rs: disposable integration evidence only; not shipped.
- installer/: per-user installation/update/uninstall.

Original C#/WPF paths and SDK installation tasks in the plan are superseded by this explicit map. Core PRD behavior remains binding except the user-approved taskbar compatibility restriction.
