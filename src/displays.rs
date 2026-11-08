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
    fn EnumWindows(callback: unsafe extern "system" fn(H, isize) -> i32, data: isize) -> i32;
    fn GetClassNameW(hwnd: H, name: *mut u16, max: i32) -> i32;
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
unsafe extern "system" fn collect_taskbar(hwnd: H, data: isize) -> i32 {
    if unsafe { IsWindowVisible(hwnd) } == 0 { return 1; }
    let mut class = [0u16; 64];
    let count = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32) } as usize;
    let class = String::from_utf16_lossy(&class[..count]);
    if class != "Shell_TrayWnd" && class != "Shell_SecondaryTrayWnd" { return 1; }
    let monitor = unsafe { MonitorFromWindow(hwnd, 2) } as usize;
    let records = unsafe { &mut *(data as *mut Vec<MonitorRecord>) };
    if let Some(record) = records.iter_mut().find(|record| record.handle == monitor) {
        if let Some(display) = record.info.as_mut() { display.taskbar = hwnd; }
    }
    1
}
pub fn list() -> Vec<Display> {
    let mut records = Vec::<MonitorRecord>::new();
    unsafe {
        EnumDisplayMonitors(std::ptr::null_mut(), std::ptr::null(), collect_monitor, &mut records as *mut _ as isize);
        EnumWindows(collect_taskbar, &mut records as *mut _ as isize);
    }
    records.into_iter().filter_map(|record| record.info).collect()
}
pub fn selected(id: &str) -> Option<Display> {
    let displays = list();
    if id.is_empty() || id == crate::layout::PRIMARY_DISPLAY { displays.iter().find(|display| display.primary).cloned() }
    else { displays.iter().find(|display| display.id == id).cloned() }
}
