# Verification Record

## Verified

- `cargo test --bin Kurmon`: settings, metric, layout, GPU overview, and interaction regression checks pass.
- The release build compiles on Windows x64 and installs as `Kurmon.exe`; the installer preserves user settings.
- Live taskbar testing confirmed CPU, GPU, RAM, and network readings, the meter right-click menu, position lock action, and settings grid editor.
- Settings preview immediately, save explicitly, and offer Save, Discard, or Cancel when closing with unsaved changes. Discard restores the previous settings.
- The overview lists detected GPU names and live utilization. Its RAM history uses a grid and filled area; the taskbar chart remains a sparkline.

## Not fully qualified

Some secondary-display, mixed-DPI, fullscreen, sleep/resume, display hotplug, high-contrast, extended endurance, and resource-budget scenarios remain untested. Do not infer full Windows configuration support from the exercised desktop checks.
