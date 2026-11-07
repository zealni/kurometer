# Product Requirements Document: Taskbar System Monitor

Status: Draft for product approval  
Working name: Taskbar System Monitor  
Document language: English  
Platform recommendation: Windows 11 desktop, x64 initially

## 1. Product summary

Taskbar System Monitor displays CPU usage, GPU usage, RAM usage, download speed, and upload speed directly in the Windows taskbar area. Users can check their computer's current workload and network activity without opening Task Manager or keeping a monitoring window on the desktop.

The product has two levels of information:

- An always-visible taskbar meter for quick checks.
- A click-to-open detail panel for device names, memory amounts, network adapter information, and explanations of unavailable readings.

Lightweight operation and a modern interface are core product requirements, not optional polish. Preserve normal taskbar interactions and run without administrator privileges for core functions. Visual effects must fit the resource budget; reduce decoration before reducing metric accuracy, accessibility, or the five requested readings.

### Requirement ownership

The user explicitly requested taskbar visibility, CPU usage, GPU usage, RAM usage, download/upload monitoring, lightweight operation, and modern design. Specific visual treatments, platform choices, numerical budgets, additional features, and release boundaries in this document are recommendations, not previously approved user decisions. Numerical performance limits are proposed acceptance targets, not measured results.

## 2. Problem and intended users

Task Manager and full monitoring dashboards provide detailed information but require users to switch context. Existing desktop widgets occupy workspace, while a tray icon alone cannot clearly show all five requested readings at once.

The initial audience is Windows users who want continuous visibility while working, downloading files, compiling software, editing media, or running other demanding applications. This is a proposed audience, not a finding from user research.

Typical questions the product should help answer:

- Is CPU or GPU activity high right now?
- How much physical memory is currently in use?
- Is a download or upload still active?
- Which GPU and network adapter does the displayed reading represent?

The application reports resource activity. High utilization alone must not be presented as proof of a bottleneck, malware, or hardware failure.

## 3. Goals and non-goals

### Goals

1. Make all five requested readings visible at a glance when the taskbar is visible and has sufficient space.
2. Keep readings understandable, including their units, selected devices, and unavailable states.
3. Preserve access to Start, pinned/running applications, Widgets, the system tray, and the clock.
4. Consume little enough CPU and memory to remain useful as a background utility.
5. Recover predictably after sleep, display changes, network changes, and Explorer restarts.

### Non-goals for the initial release

- Replacing Task Manager or providing a process manager.
- Automatically closing processes, clearing RAM, changing power settings, or tuning the system.
- Measuring internet service speed through active speed tests.
- Inspecting packets, browsing history, destinations, or application traffic content.
- Temperature, fan, voltage, or power sensors that require additional hardware-access components.
- Cloud accounts, remote monitoring, telemetry, subscriptions, advertising, or AI diagnosis.
- Support for every Windows version, architecture, taskbar replacement, or shell modification.

## 4. Taskbar integration: mandatory feasibility gate

### What counts as meeting the core requirement

In normal operation, the five readings must appear within the visible taskbar region, visually aligned with its height, without opening a separate desktop window. A standalone floating widget, a separate AppBar above the taskbar, a pinned launcher, or a tray tooltip does not independently satisfy this requirement.

A compact companion window positioned in the taskbar region is a candidate implementation, not a proven solution. The application must not require Explorer injection, replacement system files, administrator privileges, or disabling Windows security features.

### Known constraints

Microsoft documents AppBars as separate edge-anchored windows. An AppBar is not evidence that arbitrary content can be inserted into the Windows 11 taskbar [S1]. TrafficMonitor's own documentation describes overlap with Widgets and a full taskbar [S2]. These sources establish integration risk; they do not prove that this product's implementation will work.

### Required feasibility experiment

Before building the complete application, implement a disposable proof of concept with fixed sample labels and a clickable panel. Test it on the intended Windows 11 builds with:

- Centered and left-aligned taskbar icons.
- Widgets enabled and disabled.
- Sparse and crowded taskbars, including uncombined buttons where available.
- Auto-hide enabled and disabled.
- Maximized and full-screen applications.
- Primary-display changes, mixed-DPI displays, and monitor disconnection.
- Explorer restart, screen lock/unlock, and sleep/resume.
- A standard, non-administrator account.

Record the Windows build, taskbar settings, display setup, screenshots, and failures for each configuration. Declare support only for configurations that pass.

### Gate outcome

Proceed only if taskbar placement works without hiding or intercepting native controls. If reliable placement cannot be achieved under the constraints above, stop the taskbar implementation and present the evidence. A separate AppBar or floating widget requires explicit product approval as a changed requirement; it must not be silently shipped as an equivalent.

Approved scope change: the initial release supports centered/grouped taskbars with sufficient free space. Crowded or expanded-button configurations without room for all selected metrics are explicitly unsupported. Hide the meter, retain tray access with an explanation, and restore the meter when space returns. Do not cover native controls or silently replace the meter with a separate AppBar.

## 5. Release scope

P0 means required for the initial usable release. P1 means a proposed subsequent enhancement. P2 means optional expansion after the core product proves useful.

| Priority | Capability | User benefit |
| --- | --- | --- |
| P0 | CPU, GPU, RAM, download, and upload readings in the taskbar | Meets the original request |
| P0 | Detail panel with device names and reading status | Makes compact numbers interpretable |
| P0 | Explicit GPU and network adapter selection | Avoids ambiguous readings on multi-device systems |
| P0 | Show/hide and reorder metrics; select network units | Fits the information to available space |
| P0 | System/light/dark theme and DPI-aware rendering | Keeps text readable across Windows setups |
| P0 | Tray controls, optional sign-in startup, saved settings, recovery | Makes everyday background use practical |
| P1 | Rolling 60-second graphs in the detail panel | Shows whether a spike is brief or sustained |
| P1 | Optional sustained-usage alerts | Reduces the need to watch the meter continuously |
| P1 | Session download/upload totals for the selected adapter | Helps track the current transfer session |
| P2 | VRAM usage and disk activity | Adds context without crowding the default meter |
| P2 | Daily network history and export | Supports longer-term review with explicit retention rules |

P1 and P2 are developed product ideas, not commitments for the initial release. They must not delay or weaken any P0 requirement.

## 6. Primary user journeys

### First launch

1. The application starts without elevation and creates a tray entry.
2. The user sees the taskbar meter as soon as valid readings become available.
3. If multiple GPUs exist, the setup asks which to display. With one supported GPU, it selects that device.
4. The application initially selects an active physical network adapter with a default route. If there are several candidates, it asks the user to choose rather than combining traffic.
5. Startup at sign-in is offered but remains off until enabled.
6. If placement is blocked or a metric is unavailable, the detail panel and tray provide a readable explanation.

### Routine monitoring

The meter updates without moving focus away from the current application. Clicking it opens a nearby detail panel. Clicking outside or pressing Escape closes the panel. The meter does not behave like a flashing alert or animated dashboard.

### Changing what is monitored

The user opens Settings from the meter or tray, changes the selected GPU or network adapter, and sees the new device name and reading. Interval-based metrics return to a collecting state while a fresh baseline is established. Previously selected device identifiers are not reused for a different device.

### Insufficient taskbar space

The application hides the meter before it overlaps native taskbar controls. The tray entry reports that taskbar space is insufficient and still opens details and Settings. The user can disable selected metrics to reduce width. No metric is silently dropped from the chosen layout.

## 7. Functional requirements

### FR-01: Taskbar meter

- Show CPU, GPU, RAM, download, and upload by default.
- Use explicit CPU/GPU/RAM labels and distinguish download from upload with directional symbols plus accessible names.
- Display percentages as whole numbers. Reserve sufficient width for 100%.
- Show network units beside their values. Do not truncate the unit or rely on color to identify direction.
- Maintain stable geometry during ordinary value changes. Unit changes must not repeatedly shift neighboring taskbar elements.
- Allow users to hide and reorder metrics. Disallow hiding every metric while the meter is enabled; offer a separate Hide meter action instead.
- Default to the primary monitor, with selection of one other connected monitor. Simultaneous duplicate meters on multiple monitors are outside P0.

### FR-02: CPU usage

Report aggregate CPU busy time across logical processors over the sampling interval:

`CPU % = 100 × (delta total processor time - delta idle time) / delta total processor time`

The implementation must account for the Windows API's time semantics, including idle time already included in kernel time where applicable. Normalize across the whole machine, not a single logical processor. Do not average independently rounded per-core values.

Label the measurement as CPU busy time in the detail panel. Do not promise exact agreement with Task Manager, whose utilization method and sample timing may differ. Detect nonpositive intervals and counter resets instead of showing spikes or invalid percentages.

### FR-03: GPU usage

- Report utilization for one explicitly selected GPU.
- Use the busiest available engine on that GPU as the aggregate percentage, following the interpretation described by Microsoft [S3].
- Aggregate per-process activity for the same physical engine before selecting the busiest engine. Do not add different engines together or take the largest single process reading as total GPU usage.
- Identify the selected adapter by name and stable identifier. Do not assume GPU numbering matches Task Manager.
- Do not switch GPUs automatically during ordinary sampling.
- If the selected GPU disappears or Windows exposes no usable data, show an unavailable state and retain CPU, RAM, and network monitoring.
- Do not install a kernel driver or vendor SDK solely to provide P0 GPU usage.

The supported GPU/driver matrix must be established through implementation testing. Unsupported hardware is permitted only when clearly disclosed; GPU monitoring remains a required feature on the supported matrix.

### FR-04: RAM usage

Report physical memory in use:

`Used physical memory = OS-visible total physical memory - available physical memory`

`RAM % = 100 × used physical memory / OS-visible total physical memory`

The detail panel shows used and total amounts using binary units such as GiB. Do not use committed memory, page-file allocation, or the sum of process working sets as a substitute. Invalid or zero totals produce an unavailable state.

### FR-05: Download and upload meter

- Derive rates from received and transmitted byte-counter differences on one selected adapter, divided by actual elapsed monotonic time.
- Default to automatic B/s, KB/s, MB/s, and GB/s formatting using decimal units. Offer bit-rate formatting with bps, Kbps, Mbps, and Gbps, using eight bits per byte.
- Keep download and upload simultaneously visible.
- Explain in the detail panel that these are adapter receive/transmit rates. They include relevant local-network traffic and protocol overhead, not just internet file transfers.
- Do not run active speed tests, pings, or background connectivity probes.
- Let the user select a VPN or virtual adapter explicitly. Do not combine virtual and physical adapter counters, which can double-count the same traffic.
- Persist the selected adapter. If it disconnects, show Disconnected rather than silently switching to another adapter. Allow choosing another adapter in Settings.
- On reconnection, counter reset, adapter replacement, or sleep/resume, establish a fresh baseline. Never display negative rates or divide bytes accumulated during sleep by a short post-resume interval.

### FR-06: Sampling and data states

- Default sampling interval: one second. Offer two- and five-second intervals.
- Use one sampling cadence for the five readings; do not start independent loops for each visible component.
- A reading distinguishes Collecting, Valid, Unavailable, Disconnected where relevant, and Stale.
- After three expected intervals without a fresh sample, mark the reading stale. Keep its last value only in the detail panel with its age; the meter displays a nonnumeric placeholder.
- Zero is a valid measured value. It must not be used as a replacement for missing data.
- A failing metric must not freeze the UI or invalidate unrelated metrics.
- Suspending or locking the system must not create misleading post-resume samples.

### FR-07: Detail panel

Opening the meter or selecting Open details from the tray displays:

- The five readings and their units/status.
- CPU measurement definition, GPU name, RAM used/total, and selected network adapter.
- Sampling interval and age of the latest data.
- A reason when a reading is unavailable, stale, or disconnected.
- Open Task Manager and Settings actions.

Keep the panel within the selected monitor's usable bounds. It must not steal focus during background updates. Closing it stops panel-only rendering, not core sampling.

### FR-08: Settings and tray controls

Settings include visible metrics and order, selected GPU, selected network adapter, one target monitor, refresh interval, network units, theme, and startup at sign-in.

The tray menu provides Open details, Settings, Show/Hide meter, and Exit. Closing Settings does not exit the background application. Exit stops sampling, removes the tray entry, and releases any registered shell resources.

Settings are stored per user under LocalAppData, not beside an executable in a protected installation directory. Save atomically. A failed save keeps the last valid file and shows an error. Recover from invalid settings with safe defaults and a visible explanation; do not silently overwrite the invalid file.

### FR-09: Lifecycle and recovery

- Run one instance per interactive user session. A second launch opens the existing instance's details.
- Recreate the tray entry and recompute meter placement after Explorer restarts.
- Recompute layout after taskbar, theme, DPI, and display changes without requiring a restart.
- When the chosen monitor disconnects, temporarily move to the primary monitor and disclose the change. Restore the chosen monitor when it returns if placement is supported.
- Hide the meter while the taskbar auto-hides and during full-screen applications on its monitor. Restore it without taking focus.
- Startup at sign-in applies only to the current user and follows the user's explicit setting. Moving or removing the installed application must not leave a broken startup entry.

## 8. Interaction and accessibility requirements

The interface must feel modern while remaining quiet enough for continuous taskbar use. Recommended direction: Windows 11 / Fluent-inspired native surfaces, readable system typography, stable numerical columns, and restrained interaction feedback. This is a visual direction, not a requirement to adopt a heavyweight component library.

### Visual specification

| Element | Recommended treatment | Purpose |
| --- | --- | --- |
| Taskbar meter | Compact label/value groups with tabular digits and stable widths | Make all five readings easy to scan without shifting during updates |
| Detail panel | One compact surface with aligned metric rows, secondary device/status text, and a small action footer | Keep current readings primary without a full dashboard shell |
| Typography | Segoe UI Variable where available, Segoe UI fallback; use system-installed fonts | Match Windows and avoid bundled font overhead |
| Color | Theme-aware neutral surfaces and text; one Windows accent for selection and focus | Provide hierarchy without assigning decorative colors to every metric |
| Spacing | A consistent 4-DIP scale, with larger gaps between sections than within a metric row | Separate information without wasting taskbar width |
| Shape | Subtle rounded corners on the detail panel and controls; no pill around every reading | Distinguish interactive surfaces from passive numbers |
| Elevation | One subtle panel shadow; solid surfaces by default | Separate the panel from underlying applications without continuous blur |
| Motion | Immediate meter updates; only brief, user-triggered control feedback, at most 150 ms | Confirm interaction without animating resource changes |

Do not use continuous animations, rolling number transitions, decorative gradients, glow, or custom full-window blur. Respect Windows reduced-animation settings. Mica/Acrylic effects are not required for a modern appearance and must not be added unless later measurements justify their cost. Settings should use native-style controls with clear hover, pressed, disabled, and focus states.

### Accessibility and adaptation

- Follow the Windows theme by default, with working light and dark overrides.
- Support Windows high-contrast settings and at least 100%, 125%, 150%, and 200% display scaling.
- Use a solid background when transparency would make text unreadable.
- Provide accessible control names, keyboard operation, and visible focus for details and Settings. Expose tray commands through the Windows keyboard-accessible tray interface.
- Do not announce every one-second update to screen readers; announce readings when the user navigates to them.
- Escape closes transient panels. Tab and Shift+Tab follow a logical order.
- Warning states must include text or symbols rather than color alone.
- Target WCAG AA text contrast in the application's own surfaces. Validate actual rendered themes during UI implementation; no contrast result is claimed by this PRD.
- English is the initial UI language. Additional localization is not required for P0.

## 9. Proposed next-release enhancements

### Rolling history

Show the most recent 60 seconds of CPU, GPU, RAM, download, and upload in the detail panel. Keep samples only in memory. Gaps remain visible as gaps, and device changes start a new series rather than joining unrelated data. Do not draw animated graphs in the taskbar by default.

### Sustained-usage alerts

Make CPU, GPU, and RAM alerts opt-in. Proposed starting rules are usage at or above 90% for 30 consecutive seconds, one alert per sustained episode, and re-arming after usage remains below 85% for 30 seconds. Missing or stale samples break the qualifying period. Suppress notifications during full-screen use and respect Windows notification settings. Do not describe a high reading as an emergency or automatically change the system.

### Session transfer totals

Count selected-adapter received/transmitted bytes since this application session began or the user last reset the totals. Device changes start a new total. Do not describe this as an ISP billing meter. Persistent daily history requires a separate retention and reset specification before implementation.

## 10. Non-functional requirements

These are proposed release targets. Measure the complete application, including any helper process. Record the reference hardware and Windows build with results.

| Area | Proposed target | Measurement conditions |
| --- | --- | --- |
| Background CPU overhead | Mean no more than 0.5% of whole-machine CPU capacity | 10 minutes after warm-up, one-second sampling, all five metrics enabled, meter visible, detail panel closed, otherwise idle reference PC |
| Background memory | Total private bytes no more than 80 MiB | Same background scenario; include runtime and every helper process |
| First usable readings | Within 5 seconds of process start | Warm launch on the recorded reference PC, supported devices present |
| Detail panel response | Opens within 300 ms at the 95th percentile | 30 warm open/close cycles on the reference PC |
| Open-panel memory | Total private bytes no more than 100 MiB | After 30 warm detail-panel open/close cycles with all metrics enabled |
| Background storage activity | No periodic disk writes for raw samples | Observe 10 minutes with settings unchanged |
| Endurance | No crashes, frozen readings, or unbounded memory growth | 8-hour run including idle, workload, network changes, and sleep/resume |

A failure against these targets requires optimization or an explicit product decision to revise the target. It must not be reported as a measured success without evidence.

### Resource-use rules

- Use a native desktop UI. Do not ship Electron, Chromium, WebView2, a web server, or a browser-based rendering layer for this utility.
- Reuse counters and sample buffers. Do not launch shell commands, enumerate all processes, or perform full hardware discovery every sampling interval.
- Update only changed text or state. Do not drive the meter with a continuous rendering timer or request display-refresh-rate updates for one-second data.
- Create Settings and detail content on demand. Closed panels have no active rendering or animation loop. Any later history feature uses a bounded buffer, not an unbounded sample list.
- When the meter is hidden and no panel is open, stop rendering work; any retained sampling must remain within the selected cadence and resource budget. Do not busy-poll taskbar geometry.
- Measure the same budgets after opening and closing Settings, reconnecting devices, and restarting Explorer. A lean first launch alone is insufficient.
- If the recommended UI stack cannot meet the budgets after profiling, reassess the stack before requesting a higher budget. Modern styling is not an exception to the lightweight requirement.

### Privacy and security

- Core monitoring works offline without an account, telemetry, or outbound requests.
- Do not collect process lists, packet contents, hostnames, or browsing data for P0.
- Keep settings local. Diagnostic information must not automatically upload.
- Run as the current standard user. No Windows service or kernel component is required for P0.
- Review the licenses of shipped dependencies. If borrowing implementation ideas or code from existing projects, satisfy their license obligations before distribution.

## 11. Recommended implementation direction

This section records implementation recommendations rather than product requirements.

| Concern | Recommendation | Reason |
| --- | --- | --- |
| Language/runtime | Rust with the supported Windows MSVC toolchain | Native executable without a managed runtime; toolchain compilation verified locally |
| UI | Win32 native windows and controls, with custom lightweight meter drawing | Avoid browser runtimes and keep shell integration direct; modern styling still requires actual-surface verification |
| CPU/RAM | Windows system timing and memory APIs | Avoid per-sample process enumeration or shell commands |
| GPU | Windows GPU engine performance data, mapped to the selected adapter | Prefer operating-system data over additional hardware drivers |
| Network | Windows interface byte counters and monotonic elapsed time | Passive monitoring with explicit adapter semantics |
| State | A bounded latest-sample snapshot and per-user settings file | No database or service needed for P0 |
| Distribution | One per-user installer with uninstall support | Stable installation path and reliable startup registration |

Do not introduce a plugin system, backend, web frontend, persistent sample database, or generic monitoring framework for the initial release. Final API choices follow the feasibility experiment and measurement of the proposed performance targets.

## 12. Compatibility policy

The initial target is Windows 11 x64 with the standard Explorer taskbar, centered/grouped buttons, and sufficient free taskbar space. The user approved this limited scope after a live expanded-button experiment left only about 71 pixels of space. Before release, publish the exact Windows builds, display configurations, and GPU/driver combinations exercised in the compatibility matrix.

Windows 10, ARM64, Windows Server, remote-session-specific behavior, third-party taskbars, and modified Explorer environments are outside the initial supported matrix. The application must explain an unsupported environment instead of attempting invasive shell modifications.

A Windows update may change taskbar behavior. A previously working build is not evidence for a newer build. Include taskbar compatibility checks in every release qualification.

## 13. Acceptance criteria and verification

No application behavior below has been implemented or verified by this document.

| ID | Acceptance criterion | Required evidence |
| --- | --- | --- |
| AC-01 | All five readings appear inside the taskbar region on the supported standard setup | Screenshot plus interaction recording from the feasibility matrix |
| AC-02 | Start, Widgets, app buttons, tray, and clock remain unobstructed and clickable | Sparse/crowded taskbar interaction checks for both alignments; meter hides before collisions |
| AC-03 | CPU busy-time calculation handles normal work, idle, reset, and invalid intervals | Deterministic counter examples plus a live CPU workload; any Task Manager differences explained by method/timing |
| AC-04 | GPU reading represents the selected adapter's busiest aggregate engine | Deterministic engine aggregation cases plus live 3D and video/compute workloads on supported hardware |
| AC-05 | Multi-GPU selection and unavailable GPU states are explicit | Switch adapters on a multi-GPU PC; exercise missing/unsupported data without losing other readings |
| AC-06 | RAM percentage and used/total values follow the documented physical-memory formula | Comparison with the same Windows memory source and a live allocation/release scenario |
| AC-07 | Download/upload rates have correct direction, elapsed time, and units | Known counter-delta cases including bits/bytes conversion, plus an observed receive/transmit transfer |
| AC-08 | Sleep, disconnect, VPN/manual adapter selection, and counter resets do not create false transfer spikes | Recorded transition scenarios; disconnected and collecting states visible |
| AC-09 | Stale or missing data cannot masquerade as a measured zero | Stop sample delivery and fail one metric; verify its status and unaffected remaining metrics |
| AC-10 | Layout remains readable across supported themes, DPI, and monitor changes | Actual-surface screenshots, keyboard walkthrough, screen-reader check, and measured text contrast |
| AC-11 | Explorer restart and taskbar auto-hide/full-screen transitions recover without focus theft | Live shell/lifecycle walkthrough on the supported matrix |
| AC-12 | Preferences survive restart, and startup is strictly opt-in | Change preferences, restart, sign out/in, disable startup, and verify behavior as a standard user |
| AC-13 | Duplicate launch and application exit behave predictably | Launch twice; verify one sampler; exit and verify no remaining application process or shell artifacts |
| AC-14 | Performance targets and offline behavior are satisfied | Recorded resource measurements, endurance results, and network observation with no application-generated outbound traffic |
| AC-15 | Installation, update, and uninstall preserve or remove the correct state | Per-user installation walkthrough; update preserves settings; uninstall removes binaries/startup entry and offers explicit settings removal |
| AC-16 | Modern styling follows the visual specification without unnecessary rendering work | Review actual taskbar/details/Settings surfaces in light and dark themes; verify reduced-animation behavior and profile closed/hidden panels for inactive rendering loops |

### Accuracy checks

Use deterministic counter inputs to verify formulas independently from live workload timing. For live comparisons, use the same adapter, GPU, measurement definition, and interval. Establish tolerances in the verification plan from the source's resolution; do not require instantaneous equality with a differently sampled dashboard.

## 14. Delivery sequence and release gate

1. Prove taskbar placement and collision handling. Do not build around an unproven integration assumption.
2. Implement CPU, GPU, RAM, and adapter-rate collection with explicit data states.
3. Connect the meter, detail panel, settings, tray, and lifecycle recovery.
4. Complete accessibility, performance, endurance, install/update/uninstall, and compatibility checks.
5. Release only when every P0 acceptance criterion passes on the declared matrix. Evaluate P1 features afterward.

A standalone widget, missing GPU support across the entire declared matrix, fake fallback values, or a meter that covers native controls is not an acceptable initial release.

## 15. Main risks and product decisions

| Risk | Required handling |
| --- | --- |
| Taskbar placement depends on behavior that may change across Windows builds | Feasibility gate, exact supported-build matrix, no shell injection, no silent fallback |
| Too little taskbar space for five readings | User-controlled visibility/order, stable compact formatting, temporary hide with tray explanation; reject designs that hide routinely |
| GPU counters vary by hardware and driver | Exercise supported devices, validate engine aggregation, explain unavailable data |
| Adapter rates are mistaken for internet-only speed | Label the selected adapter and explain that LAN/VPN traffic may be included |
| Different CPU definitions cause comparison confusion | Document CPU busy time; do not claim exact Task Manager equivalence |
| Monitoring consumes noticeable resources | Measure all processes against the proposed budget before release |
| Additional sensor features require privileged components | Keep temperature/fan monitoring out of P0; review separately before expanding scope |

The recommendations in this document provide a default direction for review. They do not establish that the taskbar approach is feasible, that the resource budget has been met, or that the additional features have user approval.

## 16. Sources

- [S1: Microsoft Learn, Using Application Desktop Toolbars](https://learn.microsoft.com/en-us/windows/win32/shell/application-desktop-toolbars). Defines AppBars, placement negotiation, taskbar interaction, and lifecycle notifications.
- [S2: TrafficMonitor, Frequently Asked Questions](https://github.com/zhongyang219/TrafficMonitor/blob/master/Help_en-us.md). Documents Windows 11 overlap cases involving Widgets and a full taskbar. Used as implementation-risk evidence, not a dependency or a guarantee about this project.
- [S3: Microsoft DirectX Developer Blog, GPUs in the task manager](https://devblogs.microsoft.com/directx/gpus-in-the-task-manager/). Explains WDDM-based GPU data and the busiest-engine interpretation of aggregate GPU usage.
