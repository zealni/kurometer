# Verification record

## Exercised

- With the debug build, both GPUs appeared on both taskbars with NVIDIA temperature (53°C) and `--°C` for the unsupported adapter; the chosen font size rendered uniformly. A transparent empty area exposed the `KurmonHit` window as its accessibility hit target. Actual left/right clicks and locked resize gestures were not exercised because foreground input approval timed out. Existing settings were restored after the test.
- On a two-monitor desktop at 100% and 125% scaling, the debug build displayed readings on both taskbars with All displays selected and displayed only on the secondary taskbar with Secondary displays only selected. An obstructed custom position on the secondary taskbar fell back to an available gap. User preferences were restored after testing.
- Debug build on the primary taskbar: the meter remained visible while Start was open for at least two seconds. Settings opened in two columns, the display selector listed all and secondary-only choices, and the font list retained the saved Corbel preference alongside common fonts. Display choices were not saved or exercised on the secondary monitor.
- The revised uninstaller was exercised with isolated `LOCALAPPDATA`/`APPDATA` and mocked registry removal: unrelated files and subdirectories survived `-RemoveSettings`; owned files and empty directories were removed; without the flag, preferences remained.
- In an isolated preferences profile on the primary taskbar, the updated build rendered its opaque meter, RAM percentage and used GiB with a separator, and history lines. Switching to transparent mode through Settings remained visible and saved correctly. The compact two-column Settings dialog and taskbar context-menu interaction were inspected on the live desktop.
- Scan-generation and startup transaction rollback branches passed unit tests; actual registry failure injection was not performed on the user's account.
- The font-dependent experimental metric symbols were removed after they rendered ambiguously. Existing `icon_style` preferences are accepted once for migration but omitted from subsequent saves; the meter uses readable metric labels.
- The worker-to-UI snapshot handoff now replaces the displayed snapshot on every accepted sample. A regression test covers the state transition; the installed build was observed showing live CPU, three GPU, RAM, download, and upload values after restart.
- The meter now uses DPI-scaled GDI vector icons, muted labels, semibold values, and width fitting that accounts for icon and bold-text width. The overview groups summary and source status in neutral cards; the settings panel retains native named controls and keyboard behavior.

- The installed running executable reports FileDescription and ProductName `Kurmon` through Windows file-version metadata. The Task Manager window was not readable through the desktop capture during verification, so its displayed search result remains unverified.

- With a temporary preferences directory, observed all five history charts in the details panel and CPU/GPU/RAM/download/upload values with small charts on the transparent taskbar meter; RAM showed both percentage and used GiB. The Windows font selector exposed installed families including Arial Narrow and Cascadia Code. Saving a changed CPU chart checkbox persisted `charts=01111`.
- Red meter text contrast: #FF6B6B on #202020 is 5.87:1 and #C42B1C on #F5F5F5 is 5.19:1, both passing WCAG AA normal text.

- Packaged the current release executable with `install.ps1`, README and logo. Extracted the ZIP into a temporary profile and exercised install, upgrade with preferences retained, uninstall with preferences retained, and uninstall with `-RemoveSettings`. Registry removal was mocked to avoid modifying the real account.
- The packaged executable's `--sample` command returned live CPU, RAM, GPU and network samples; `cargo test --bin taskbar-system-monitor` passed 20 tests.

- Rust MSVC compilation and optimized release build on the local Windows x64 desktop.
 - `cargo test`: 18 checks across two binary suites passed, including adapter-name/font/appearance preference round-trips; CPU timing, network rate, GPU aggregation, placement boundaries, and shell-event filtering.
- Live native CPU, physical RAM, GPU engine, Wi-Fi receive/transmit samples observed.
- Real taskbar meter click opens details; Settings exposes named native controls and named Intel Iris Xe GPU selection through accessibility.
- Bits/bytes preference changed and saved through actual Settings UI.
- Invalid order was rejected with a readable dialog; valid reordered metrics persisted when Settings reopened, then defaults were restored.
- A screenshot exposed black checkbox labels on dark background; the following build disabled checkbox theme painting and fresh screenshots confirmed readable light labels.
- Probe: Explorer restart, auto-hide hide/restore, crowded expanded-button hide behavior exercised. This evidence is not automatically proof for every subsequent application build.
- Dark text pairing #F4F4F4 on #202020: contrast checker returned 14.81:1, passing normal text.
- Early release 30.1-second observation: 8.54 MiB private bytes, 0.016% mean whole-machine CPU, executable 296 KiB. This was not the required idle-reference ten-minute benchmark and predates subsequent lifecycle changes.
- Worker-based release: 30-second observation returned 8.94 MiB private bytes, 0.013% whole-machine CPU and 330 KiB executable. This is a short observation, not performance qualification.
- Sampling now runs outside the UI thread with a bounded result channel. Live readings remained visible in the worker-based release.
- Escape closed the overview, confirmed by an empty matching-window list. Final Settings screenshot shows readable checkbox labels and no clipped support text.
- Display selection saved `\\.\DISPLAY1` to preferences and was restored after rebuilding and restarting the application.
- Portable package refreshed from the final primary-taskbar release at `dist/taskbar-system-monitor-windows-x64.zip` (179,259 bytes; executable 350,720 bytes plus README.md).
- Full installer lifecycle tested in isolated PowerShell sandbox: fresh install, update with preserved `settings.ini`, clean uninstall keeping settings, and uninstall with `-RemoveSettings` deleting the settings directory.
- Fluent polish release: DWM dark titlebar and rounded corners, owner-drawn rounded buttons, stable fixed-slot meter and cyan network arrows verified on live Windows desktop.
- Dark meter contrasts: #A0A0A0 labels 6.23:1 and #4CC2FF arrows 8.12:1 against #202020; both pass normal text AA.
- Saved GPU changed from volatile LUID to adapter display name. Confirmed an old saved LUID self-healed, name saved as `Intel(R) Iris(R) Xe Graphics`, then application restart continued displaying live GPU utilization.
- Dropdowns remain native white controls. DarkMode_Explorer had no visible effect; custom-draw experiment hid selected values and was reverted. Functional settings labels/selected values confirmed on screenshot and AX.
- Final release on the live desktop: meter is on the primary taskbar at x=8, y=1034, width=571, height=44; taskbar is 48px high, leaving a 2px inset at top and bottom. Five metrics render on one row without clipping.
- Primary-only monitor discovery verified by Settings exposing only `\\.\DISPLAY5` despite a previously saved secondary display ID; meter remained at primary taskbar coordinates.
- Transparent-background color-key and Amber palette verified on the live taskbar; metric text stays opaque. Green (#6CCB8A) 8.18:1 and Amber (#F0B44D) 8.80:1 against #202020 pass normal-text AA.
 - Font options Segoe UI, Arial and Consolas are radio controls. Consolas was selected, saved as `font=Consolas`, and displayed checked when Settings reopened.
 - Settings exposes transparent background, Blue/Green/Amber palette, taskbar alignment, position lock, and font selection; transparency plus Amber was saved and visible on the live meter.
- The earlier Explorer-hosted experiment (attaching to the XAML content bridge) was withdrawn after taskbar disappearance. Windows Application events on 2026-11-07 at 00:39 recorded `AppHangXProcB1` for `explorer.exe` naming `taskbar-system-monitor.exe`, and Kurmon access violations (`0xc0000005`) in `IMM32.DLL`. A later implementation instead parents the meter directly to `Shell_TrayWnd` (TrafficMonitor's approach), keeps panels and the tray icon on a separate hidden top-level controller, moves UI Automation to a windowless MTA worker, and removes the meter's IME context with `ImmAssociateContextEx`.
- Live check of that implementation: the accessibility tree lists a `Kurmon` group as a child of the taskbar and no top-level meter window exists; readings stayed visible while Explorer's taskbar menu and the Start menu opened beside it; Explorer remained responsive; killing and restarting Explorer reattached the meter to the new taskbar automatically.
- Clicking the hosted meter opened the overview panel, which is a separate top-level window owned by the controller rather than a taskbar child. Escape close had been verified with the earlier overlay build; the post-hosting attempt returned an invalid-window-handle error because the panel was already gone.
- Observed `SetParent` refusal: while the shell was busy, `SetParent` returned failure with `ERROR_INVALID_PARAMETER` (87) and the process stayed a topmost overlay. The app now retries up to six times at 5-second intervals and clears that state on shell events; one such overlay instance only became embedded after an Explorer restart. Not every refusal is transient enough to recover without restarting Explorer.

## Not qualified yet

The application must not be described as PRD-complete. Outstanding implementation/qualification includes:

- Display selection supports Primary, All, secondary-only, and named taskbar-backed monitors. Explorer taskbar recreation, hotplug reconciliation, and other monitor layouts remain unverified. The two-monitor check above does not establish compatibility with all mixed-DPI configurations.
- Taskbar activation investigation: worktree builds ran temporarily with a copy of the user's settings. Six screenshots after minimizing Brave and twelve after a taskbar click showed meter text, sampled roughly 100 ms apart. Brave was restored through the taskbar's accessibility invoke action; the restore transition itself was not captured. The `main` build containing the overlay-recreation change was subsequently installed, verified against the release executable by SHA-256, and observed on the primary taskbar. Flicker shorter than the sampling interval and a full restore cycle remain unverified.
- The independent overlay fallback can still blink when taskbar activation changes window stacking order. Hosted mode was not exercised with a pinned-app jump list open, only with Explorer's taskbar menu and the Start menu.
- Previous builds had an invisible hosted meter with transparent background disabled. The revised opaque layered mode rendered on this desktop; other Windows configurations remain unqualified.
- Full-screen handling is implemented but not exercised; screen-lock lifecycle remains unqualified.
 - Accessible focus-based reading announcements, high-contrast and 200% DPI verification; detail panel remains a classic text layout.
- Explicit per-metric stale age semantics and stalled-worker recovery checks.
- GPU multi-device choice on a real multi-GPU workload, selected-adapter default-route ambiguity handling, hot-unplug/reconnect scenarios.
 - Interactive resize drag was attempted but desktop input fell back to takeover and failed; drag resize and unlocked movement remain unverified, and hosted mode disables them entirely.
- Live startup opt-in sign-in validation on user account (not triggered during testing to avoid mutating user profile).
- Ten-minute resource budget, panel latency distribution, outbound traffic/disk-write observation, eight-hour endurance.

