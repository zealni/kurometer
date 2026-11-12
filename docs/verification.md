# Verification Record

## Verified

- `cargo test --bin Kurmon`: settings, metric, layout, GPU overview, and interaction regression checks pass.
- The release build compiles on Windows x64 and installs as `Kurmon.exe`; the installer preserves user settings.
- Live taskbar testing confirmed CPU, GPU, RAM, and network readings, the meter right-click menu, position lock action, and settings grid editor.
- Live Windows 11 desktop verification confirmed primary and secondary taskbar meters stay visible while the Start menu is open (`Shell_TrayWnd` in `ZBID_SYSTEM_TOOLS` band 6), and regression checks confirm `StartMenuExperienceHost.exe` and `SearchHost.exe` do not trigger fullscreen hiding.
- Settings preview immediately, save explicitly, and offer Save, Discard, or Cancel when closing with unsaved changes. Discard restores the previous settings.
- The overview lists detected GPU names and live utilization. Its RAM history uses a grid and filled area; the taskbar chart remains a sparkline.

## Not fully qualified

- Behavior across other Windows builds and untested secondary-display, mixed-DPI, fullscreen-app, sleep/resume, display hotplug, high-contrast, endurance, and resource-budget scenarios remains unqualified. Do not infer full Windows configuration support from the exercised desktop checks.
