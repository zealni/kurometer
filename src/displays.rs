use std::{ffi::c_void, ptr::null};
type H = *mut c_void;
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
#[repr(C)]
struct MonitorInfo {
    size: u32,
    monitor: Rect,
    work: Rect,
    flags: u32,
    device: [u16; 32],
}
#[link(name = "user32")]
unsafe extern "system" {
    fn FindWindowW(class: *const u16, title: *const u16) -> H;
    fn MonitorFromWindow(hwnd: H, flags: u32) -> H;
    fn GetMonitorInfoW(monitor: H, info: *mut MonitorInfo) -> i32;
}
#[derive(Clone)]
pub struct Display {
    pub id: String,
    pub taskbar: H,
    pub screen: Rect,
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub fn list() -> Vec<Display> {
    unsafe {
        let taskbar = FindWindowW(wide("Shell_TrayWnd").as_ptr(), null());
        if taskbar.is_null() {
            return vec![];
        }
        let monitor = MonitorFromWindow(taskbar, 1);
        let mut info = MonitorInfo {
            size: std::mem::size_of::<MonitorInfo>() as u32,
            monitor: Default::default(),
            work: Default::default(),
            flags: 0,
            device: [0; 32],
        };
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            return vec![];
        }
        vec![Display {
            id: String::from_utf16_lossy(
                &info.device[..info.device.iter().position(|c| *c == 0).unwrap_or(32)],
            ),
            taskbar,
            screen: info.monitor,
        }]
    }
}
pub fn selected(_id: &str) -> Option<Display> {
    list().into_iter().next()
}
