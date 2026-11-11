# Taskbar Integration Feasibility

## Findings

The placement probe showed that native child-window bounds are unreliable. Kurmon uses UI Automation to locate taskbar controls. A crowded taskbar may have no safe space for the meter, so Kurmon hides it rather than covering native controls.

The meter is hosted under `Shell_TrayWnd`; panels and the notification icon remain on a separate controller. UI Automation placement scans run off the UI thread. Explorer may temporarily refuse parenting, so Kurmon retries and can use an overlay fallback.

## Remaining qualification

Explorer restart recovery, mixed-DPI changes, display disconnect/reconnect, sleep/resume, fullscreen handling and some taskbar layouts still need broader testing. Hosted meters cannot be freely dragged or resized; use Settings. Placement and compatibility claims should remain limited to scenarios that have been exercised.
