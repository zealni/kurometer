use std::ffi::c_void;
type H = *mut c_void;
#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Rect { pub left: i32, pub top: i32, pub right: i32, pub bottom: i32 }
#[repr(C)]
struct MonitorInfo { size: u32, monitor: Rect, work: Rect, flags: u32, device: [u16; 32] }
#[link(name = "user32")]
unsafe extern "system" {
    fn EnumDisplayMonitors(dc: H, clip: *const Rect, callback: unsafe extern "system" fn(H, H, *mut Rect, isize) -> i32, data: isize) -> i32;
    fn FindWindowExW(parent: H, after: H, class: *const u16, title: *const u16) -> H;
    fn MonitorFromWindow(hwnd: H, flags: u32) -> H;
    fn GetMonitorInfoW(monitor: H, info: *mut MonitorInfo) -> i32;
    fn IsWindowVisible(hwnd: H) -> i32;
}
#[derive(Clone)]
pub struct Display { pub id: String, pub taskbar: H, pub screen: Rect, pub work: Rect, pub primary: bool }
#[derive(Default)]
struct MonitorRecord { handle: usize, info: Option<Display> }
unsafe extern "system" fn collect_monitor(monitor: H, _: H, _: *mut Rect, data: isize) -> i32 {
    let records = unsafe { &mut *(data as *mut Vec<MonitorRecord>) };
    let mut info = MonitorInfo { size: std::mem::size_of::<MonitorInfo>() as u32, monitor: Rect::default(), work: Rect::default(), flags: 0, device: [0; 32] };
    if unsafe { GetMonitorInfoW(monitor, &mut info) } != 0 {
        let end = info.device.iter().position(|c| *c == 0).unwrap_or(info.device.len());
        records.push(MonitorRecord { handle: monitor as usize, info: Some(Display {
            id: String::from_utf16_lossy(&info.device[..end]), taskbar: std::ptr::null_mut(),
            screen: info.monitor, work: info.work, primary: info.flags & 1 != 0,
        }) });
    }
    1
}
fn collect_taskbar_class(class: *const u16, records: &mut [MonitorRecord]) {
    let mut after = std::ptr::null_mut();
    let mut seen = [std::ptr::null_mut(); 64];
    for count in 0..seen.len() {
        let hwnd = unsafe { FindWindowExW(std::ptr::null_mut(), after, class, std::ptr::null()) };
        if hwnd.is_null() || seen[..count].contains(&hwnd) {
            break;
        }
        seen[count] = hwnd;
        after = hwnd;
        if unsafe { IsWindowVisible(hwnd) } == 0 {
            continue;
        }
        let monitor = unsafe { MonitorFromWindow(hwnd, 2) } as usize;
        if let Some(record) = records.iter_mut().find(|record| record.handle == monitor) {
            if let Some(display) = record.info.as_mut() {
                display.taskbar = hwnd;
            }
        }
    }
}
pub fn list() -> Vec<Display> {
    let mut records = Vec::<MonitorRecord>::new();
    unsafe {
        EnumDisplayMonitors(std::ptr::null_mut(), std::ptr::null(), collect_monitor, &mut records as *mut _ as isize);
    }
    // EnumWindows skips ZBID_SYSTEM_TOOLS (band 6) taskbars when Start or Search is open.
    collect_taskbar_class(windows::core::w!("Shell_TrayWnd").as_ptr(), &mut records);
    collect_taskbar_class(windows::core::w!("Shell_SecondaryTrayWnd").as_ptr(), &mut records);
    records.into_iter().filter_map(|record| record.info).collect()
}
pub fn selected(id: &str) -> Option<Display> {
    let displays = list();
    if id.is_empty() || id == crate::layout::PRIMARY_DISPLAY { displays.iter().find(|display| display.primary).cloned() }
    else { displays.iter().find(|display| display.id == id).cloned() }
}

pub fn at_point(displays: &[Display], point: [i32; 2]) -> Option<&Display> {
    displays.iter().find(|d| point[0] >= d.screen.left && point[0] < d.screen.right
        && point[1] >= d.screen.top && point[1] < d.screen.bottom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taskbar_click_finds_display_outside_its_work_area() {
        let display = Display {
            id: "primary".into(), taskbar: std::ptr::null_mut(),
            screen: Rect { left: 0, top: 0, right: 1920, bottom: 1080 },
            work: Rect { left: 0, top: 0, right: 1920, bottom: 1032 },
            primary: true,
        };
        assert_eq!(at_point(&[display], [100, 1050]).unwrap().work.bottom, 1032);
    }

    #[test]
    fn list_includes_visible_primary_taskbar_when_present() {
        let bar = unsafe { FindWindowExW(std::ptr::null_mut(), std::ptr::null_mut(), windows::core::w!("Shell_TrayWnd").as_ptr(), std::ptr::null()) };
        if !bar.is_null() && unsafe { IsWindowVisible(bar) } != 0 {
            assert!(list().iter().any(|display| display.taskbar == bar));
        }
    }
}
