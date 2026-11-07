# Taskbar System Monitor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver the P0 Windows taskbar monitor described in PRD.md, subject to its mandatory taskbar feasibility gate.

**Architecture:** A single per-user native desktop process samples Windows counters and publishes timestamped readings to a compact taskbar-region window and on-demand detail/settings windows. Keep taskbar placement isolated from collection so shell failures do not corrupt readings. Do not inject into Explorer or substitute a floating widget for the core requirement.

**Tech Stack:** Recommended C#, supported .NET LTS, WPF, Win32 interop, Windows GPU performance counters. The machine currently has no dotnet SDK in PATH or the standard Program Files location; a legacy .NET Framework C# compiler exists and may build a disposable Win32 placement probe without installing tooling.

**Spec:** ../../../../PRD.md

## Global Constraints

- Windows 11 x64 with the standard Explorer taskbar initially; publish tested builds/configurations only.
- CPU, GPU, RAM, download, and upload visible in the taskbar region; no Explorer injection, driver, elevation, or browser runtime.
- Default one-second sampling; options two and five seconds. Mark stale after three expected intervals.
- Proposed budgets: average background CPU at most 0.5% of whole-machine capacity; total private bytes at most 80 MiB background and 100 MiB after panel use.
- Modern Windows 11 / Fluent-inspired styling, system fonts, theme adaptation, keyboard access, no continuous animation or heavy blur.
- No telemetry, cloud service, or active network probes. Startup is opt-in.
- Implement P0 only; do not add P1/P2 functionality to compensate for incomplete core behavior.

## Review Focus

1. Crowded taskbar and Widgets: never cover or intercept native controls; expose why the meter is hidden.
2. GPU engine instances: aggregate processes within each physical engine, then choose the busiest engine of the selected adapter.
3. Counter resets and resume: establish a new baseline rather than showing artificial transfer spikes.
4. Display disconnection and Explorer restart: recover placement/tray without stealing focus.
5. Invalid settings and failed writes: preserve the last valid file and explain recovery rather than destroying user preferences.

## File structure

- `tools/TaskbarProbe.cs`: disposable placement and interaction experiment, no real readings.
- `docs/taskbar-feasibility.md`: actual build/display/taskbar evidence and gate decision.
- `src/TaskbarMonitor/TaskbarMonitor.csproj`, `App.xaml`, `App.xaml.cs`: application entry, single-instance behavior, shared styles, shutdown.
- `src/TaskbarMonitor/NativeMethods.cs`: narrowly scoped Windows interop.
- `src/TaskbarMonitor/Metrics.cs`: reading states, interval calculations, CPU/RAM/network sampling.
- `src/TaskbarMonitor/GpuSampler.cs`: GPU discovery, stable selection, per-engine utilization.
- `src/TaskbarMonitor/TaskbarWindow.cs`: placement, collision checks, DPI/shell events, meter rendering.
- `src/TaskbarMonitor/DetailsWindow.xaml` and `.xaml.cs`: on-demand live details.
- `src/TaskbarMonitor/SettingsWindow.xaml` and `.xaml.cs`: validated preferences with native-style controls.
- `src/TaskbarMonitor/UserSettings.cs`: per-user atomic persistence and startup registration.
- `tests/TaskbarMonitor.Checks/`: dependency-light assert-based behavioral checks for metric arithmetic and settings failure paths.
- `installer/`: per-user installer/update/uninstall implementation after feasibility passes.
- `README.md`, `CHANGELOG.md`: build/run instructions, tested support matrix, limitations, release changes.

### Task 1: Prove taskbar feasibility

**Produces:** documented go/no-go result, not a product scaffold.

- [ ] Implement the disposable probe with fixed CPU/GPU/RAM/download/upload labels, a clickable details window, and no shell injection.
- [ ] Compile using the available Windows compiler; launch it and observe its actual surface through desktop screenshots/accessibility.
- [ ] Verify native Start, app buttons, Widgets, tray, and clock remain accessible. Include explicit sparse/crowded geometry checks; hiding is preferable to overlap.
- [ ] Exercise non-disruptive local configurations first. Do not restart Explorer, alter user taskbar/display settings, or interrupt full-screen work without permission; record unexercised scenarios rather than claiming support.
- [ ] Record actual Windows build, DPI, monitor geometry, alignment, screenshots, and findings in `docs/taskbar-feasibility.md`.
- [ ] If reliable collision-free placement cannot be demonstrated, stop product implementation and request a product decision with evidence, as required by PRD section 4.

### Task 2: Establish toolchain and metric behavior

**Consumes:** passing feasibility gate. **Produces:** timestamped CPU/GPU/RAM/network readings with explicit states.

- [ ] Obtain a supported .NET SDK using an official source with user approval for installation; verify `dotnet --info`. Do not silently change to an unsupported runtime.
- [ ] Create the desktop project and a minimal assertion executable for arithmetic, not UI wiring.
- [ ] Add failing checks for CPU time semantics, invalid elapsed time, RAM total zero, byte/bit conversions, negative counter deltas, reconnect/resume baselines, and three-interval stale transitions.
- [ ] Implement reusable sampling buffers, monotonic elapsed time, and isolated metric failures. Run checks and observe live idle/CPU/network workloads.
- [ ] Add failing checks proving process contributions within one GPU engine are aggregated and distinct engines are not summed. Implement stable adapter mapping, live GPU selection, and unavailable states.
- [ ] Exercise real GPU workloads where available; record exact driver/device coverage. No mock-only claim of GPU support.

### Task 3: Integrate modern UI, settings, and lifecycle

**Consumes:** validated snapshots and proven placement approach. **Produces:** complete interactive P0 utility.

- [ ] Implement the meter with stable-width tabular figures, five default readings, explicit units, changed-value-only rendering, and accessible labels.
- [ ] Implement on-demand details and Settings with system typography, neutral theme-aware surfaces, restrained accent, keyboard focus, reduced-animation support, and high-contrast adaptation.
- [ ] Implement device/monitor selection, metric visibility/order, refresh intervals, units, themes, startup opt-in, tray commands, single-instance behavior, and full exit cleanup.
- [ ] Write failing settings checks for corrupt input and failed atomic replacement; implement validated LocalAppData persistence preserving the invalid/last valid file as specified.
- [ ] Implement Explorer/tray recovery, monitor fallback/restoration, full-screen and auto-hide behavior, and no-focus background updates. Exercise actual surface transitions and document unavailable scenarios.
- [ ] Profile all processes with five readings enabled, then after repeated Settings/details use. Stop hidden/closed-panel rendering and unnecessary discovery work; if budgets fail, profile before reconsidering the stack.

### Task 4: Package and qualify the P0 release

**Produces:** installable application and recorded verification, not an unverified release claim.

- [ ] Implement per-user install/update/uninstall with stable paths, settings-preserving updates, startup cleanup, and explicit settings-removal choice.
- [ ] Run behavioral checks once after integration and launch the installed application. Verify AC-01 through AC-16 with recorded evidence.
- [ ] Check light/dark/high contrast, supported DPI values, keyboard/screen-reader interaction, and actual color contrast.
- [ ] Measure CPU, private bytes, startup, panel latency, disk writes, and outbound traffic under the PRD conditions. Complete the eight-hour endurance scenario before declaring release qualification.
- [ ] Update README/CHANGELOG and the exact tested compatibility matrix. Remove the disposable probe after its findings are preserved.
- [ ] Report passing and unexercised criteria separately. Do not label an incomplete qualification or failed feasibility gate a finished application.

## Plan review

The tasks cover PRD P0 scope, all 16 acceptance criteria, deployment, and the explicit lightweight/modern requirements. P1/P2 remain excluded. The first milestone intentionally blocks later work if the defining taskbar requirement cannot be met safely. Toolchain installation and disruptive desktop verification are explicit approval points, not reasons to fabricate success.
