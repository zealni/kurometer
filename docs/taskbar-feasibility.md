# Taskbar integration feasibility record

Status: Gate not yet passed. This is not a finished monitoring application.

## Observed evidence

- Installed Rust: rustc 1.99.1, host x86_64-pc-windows-msvc; Cargo 1.99.1.
- `cargo build --bin taskbar-probe` completed successfully without external crates.
- `target/debug/taskbar-probe.exe --inspect` observed the primary Explorer taskbar at (0, 1020)-(1920, 1080).
- Native child rectangles included (0, 1020)-(65535, 66555) and a full-taskbar rectangle. Naive child-window bounding boxes are therefore insufficient for safe collision detection on this desktop.
- Windows accessibility exposed centered Start/application buttons and the tray/clock controls.
- The disposable probe displayed at (12, 1020), size 390 by 60, in the currently empty left-hand taskbar region.
- A real window screenshot showed the two-line fixed sample labels. Clicking the probe opened the sample-data disclosure dialog, confirmed through UI Automation.
- The dialog disappeared after its OK action, although the automation call reported an AX event error. The probe remained present in the subsequent window list.
- A background Alt+F4 attempt could not activate the no-activate probe window; no keyboard input was delivered. The experiment has an automatic 120-second shutdown timer.
- Added official `windows` crate bindings for UI Automation. Live discovery returned actual button rectangles, avoiding the invalid native child bounds.
- Added four deterministic placement checks. Centered and left-aligned gap cases failed before implementation, then all four passed, including crowded and overlapping-region cases.
- Live semantic discovery selected x=8 with width=390 on the current centered taskbar.
- With explicit user permission, changed alignment and auto-hide through Settings and restarted Explorer. Restored Center alignment and unchecked Automatically hide the taskbar, confirmed by a fresh screenshot.
- The Left setting changed in Settings, but the observed Start button remained at x=602; actual left-aligned shell behavior is not proven by that setting change.
- The running probe remained visible at taskbar height after Explorer restarted, verified through a fresh window screenshot.
- The initial auto-hide test moved the probe to y=1078 instead of hiding it cleanly. After the offscreen check, a live retest hid the probe and restored it to (8,1020) when auto-hide was disabled. Both UI observation and the completed probe log confirmed this transition.
- With taskbar grouping set temporarily to Never, actual buttons occupied x=0 through x=1497, with the tray beginning at x=1568. No 390-pixel gap existed; live discovery and shell-event handling both reported HIDDEN instead of covering controls. Restored grouping to Always, confirmed through Settings.
- Shell transition logs contained temporarily inconsistent overlapping button rectangles. Therefore one accessibility snapshot cannot be assumed to describe a settled layout.

## Limits

The probe uses measured accessible button rectangles for initial gap selection, but still shows sample values rather than measured system readings. It is disposable, not a production-safe placement algorithm. GPU collection, settings persistence, installer, and modern application UI are not implemented.

Still unexercised: Widgets changes, sleep/resume, display disconnection, mixed-DPI transitions, and a settled left-aligned layout independent of the expanded-button case. Settings automation sometimes reported pointer-restoration errors after delivering a click; screenshots were inspected before taking further action.

## Gate decision

The current approach fails the full gate: the actual expanded-button configuration leaves only about 71 pixels before the tray, insufficient for all five readable values. Frequent hiding on that normal configuration is explicitly disallowed by the PRD. Making a 390-pixel meter somewhat narrower does not solve a 71-pixel gap without dropping information or readability. Reserving space inside the native taskbar has not been demonstrated under the no-injection constraint.

Stop product implementation at this gate as required by PRD section 4. A product decision is needed: accept an explicitly limited initial compatibility scope (centered/grouped taskbar with sufficient space), or change the core placement requirement to allow a separate reserved AppBar. Neither is silently treated as equivalent to the approved PRD.

## Stack ruling

Use Rust plus native Win32 rather than C#/WPF for the intended implementation. The user raised Rust as an alternative, it fits the no-browser/lightweight goal, and the installed native toolchain successfully compiled the probe. Rust does not remove Windows taskbar compatibility risk. The original implementation plan's C#/WPF file map and SDK installation steps are superseded; update its downstream task details only after the taskbar gate passes, rather than treating an unverified integration as a product scaffold.
