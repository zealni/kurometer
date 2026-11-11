# Kurmon

![Kurmon logo](kurmon.svg)

Kurmon (Kuro Monitor) is a native Rust and Win32 system monitor for the Windows 11 taskbar. It displays CPU busy time, GPU utilization, physical RAM usage, and receive/transmit rates for a selected network adapter. The repository is named `kurometer`.

## Build and run

On Windows x64, install the Rust MSVC toolchain, Windows C++ build tools, and Windows SDK (including the resource compiler). From the repository root, run:

```powershell
cargo build --release --bin taskbar-system-monitor
.\target\release\taskbar-system-monitor.exe
```

The executable retains its existing name to preserve compatibility. Its Windows file description is **Kurmon**, which identifies it in Task Manager; the Details tab still shows `taskbar-system-monitor.exe`. Launching it again opens the existing instance. Avoid enabling startup for a temporary build path that you may later move.

## Controls and settings

Left-click the meter or its notification-area icon to open the details panel. Right-click either for details, settings, show/hide, and exit. The details panel also opens settings and Task Manager. Details and settings open beside the meter, within its display's work area. Press Escape to close a panel.

With a transparent background, the meter's whole reserved area accepts clicks. Taskbar controls outside that area remain accessible.

Settings include the network and GPU adapters, refresh interval, bits or bytes, theme, font, transparent background, color palette, meter width and positioning, position lock, visible readings and their order, and optional sign-in startup. Display selection offers the primary taskbar, all taskbar-backed displays, secondary taskbar-backed displays only, or one named display. Settings opens as a scrollable panel near the selected taskbar. In the fallback overlay mode, positioning unlocked allows moving the meter and dragging its right edge to resize it; while hosted by the taskbar those gestures are unavailable, so use Settings.

Choose which metrics, labels, and history charts appear individually. RAM can show percentage, used GiB, or both (for example, `72% · 11.4 GiB`). CPU, GPU, and RAM readings turn red at 95% or above. The font selector offers common installed fonts and preserves an existing preference; the size selector sets a maximum that shrinks uniformly when needed to fit. Settings groups reading sources and taskbar display controls in two columns when the screen is wide enough and scrolls on smaller displays. The overview aligns readings and values in separate columns.

Enable **Show all GPUs** to add a separate meter for each detected adapter. **GPU temperature** uses the installed NVIDIA CUDA and NVML drivers when available; unsupported or inaccessible sensors show `--°C`. CPU and non-NVIDIA GPU temperatures are not provided. Additional GPU meters do not have separate history charts.

Enabled charts appear beside taskbar readings and in the details panel. History retains the latest 60 samples in memory, at the selected refresh interval, and resets when Kurmon exits. Network charts scale to the maximum observed rate; they do not represent network capacity.

Meter cells and text adapt to the available taskbar height, width, enabled metrics, and display scale. Width and custom offset are stored as 96-DPI logical pixels; existing values saved by older versions at a different scale are not migrated because their original DPI is unknown. Automatic placement shrinks the meter to an available gap. On secondary displays, an unsafe preferred position falls back to automatic placement; if the layout cannot fit safely, the meter hides rather than covering taskbar controls.

## Installation

Download and extract `dist/taskbar-system-monitor-windows-x64.zip`, open PowerShell in the extracted folder, and run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

The package contains the release executable; Rust is not needed to install it. The installer copies the executable and uninstall script to `%LOCALAPPDATA%\Programs\TaskbarSystemMonitor` and creates a Kurmon Start menu shortcut. It does not enable startup. Exit the installed application before updating or uninstalling. An existing Kurmon shortcut pointing elsewhere is not overwritten. Older Taskbar System Monitor shortcuts are left untouched.

From a source checkout, build first with `cargo build --release --bin taskbar-system-monitor`, then run `powershell -NoProfile -ExecutionPolicy Bypass -File .\installer\install.ps1`.

To uninstall while retaining preferences, or to remove them explicitly:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File "$env:LOCALAPPDATA\Programs\TaskbarSystemMonitor\install.ps1" -Uninstall
# Alternatively, remove preferences too:
powershell -NoProfile -ExecutionPolicy Bypass -File "$env:LOCALAPPDATA\Programs\TaskbarSystemMonitor\install.ps1" -Uninstall -RemoveSettings
```

Preferences remain at `%LOCALAPPDATA%\TaskbarSystemMonitor\settings.ini` unless removed explicitly. Uninstall removes only the installed executable and installer; `-RemoveSettings` also removes Kurmon's `settings.ini` and `settings.tmp`. Other files and non-empty directories are preserved. The executable, installation directory, settings directory, and startup registry value keep their existing identities.

## Placement and limitations

Kurmon hosts the meter as a child window of the primary Windows 11 taskbar (`Shell_TrayWnd`), the placement approach used by TrafficMonitor. Explorer can transiently refuse cross-process parenting while the shell is busy; in that case Kurmon runs as a topmost companion window and retries on later shell events. Panels and the notification-area icon stay on a separate top-level controller window, so they never become children of Explorer.

Placement avoids taskbar buttons: occupied rectangles are read with UI Automation on a worker thread (never the UI thread), and the meter hides rather than covering controls. Only controls intersecting the taskbar count as occupied space. Crowded or expanded taskbars are not supported; the notification-area icon remains available. The meter also hides for fullscreen windows. While hosted, taskbar dragging and resizing are unavailable because a child of Explorer cannot start a caption move; set width and position in Settings instead. Mixed-DPI and multi-monitor behavior are not qualified.

Network rates include LAN traffic on the selected adapter and are not an internet speed test. GPU utilization represents the busiest aggregate engine on the selected adapter; CPU busy time can differ from Task Manager's utilization method. Unavailable readings show `--`.

See `docs/verification.md` for the recorded verification and remaining qualification gaps, including fullscreen handling, accessibility, startup at sign-in, and resize/movement gestures.
