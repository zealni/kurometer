// Disposable feasibility experiment: sample labels are not measured readings.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{ffi::c_void, mem::size_of, ptr::null_mut};
type Handle = *mut c_void;
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct Class {
    style: u32,
    proc: Option<unsafe extern "system" fn(Handle, u32, usize, isize) -> isize>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu: *const u16,
    name: *const u16,
}
#[repr(C)]
#[derive(Default)]
struct Point {
    x: i32,
    y: i32,
}
#[repr(C)]
struct Message {
    hwnd: Handle,
    message: u32,
    wp: usize,
    lp: isize,
    time: u32,
    point: Point,
    private: u32,
}
#[repr(C)]
struct Paint {
    dc: Handle,
    erase: i32,
    rect: Rect,
    restore: i32,
    update: i32,
    reserved: [u8; 32],
}
#[link(name = "user32")]
unsafe extern "system" {
    fn FindWindowW(class: *const u16, name: *const u16) -> Handle;
    fn FindWindowExW(parent: Handle, after: Handle, class: *const u16, name: *const u16) -> Handle;
    fn GetWindowRect(hwnd: Handle, rect: *mut Rect) -> i32;
    fn RegisterClassW(class: *const Class) -> u16;
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: Handle,
        menu: Handle,
        instance: Handle,
        param: Handle,
    ) -> Handle;
    fn DefWindowProcW(hwnd: Handle, msg: u32, wp: usize, lp: isize) -> isize;
    fn ShowWindow(hwnd: Handle, cmd: i32) -> i32;
    fn SetWindowPos(hwnd: Handle, after: Handle, x: i32, y: i32, w: i32, h: i32, flags: u32)
    -> i32;
    fn GetMessageW(msg: *mut Message, hwnd: Handle, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const Message) -> i32;
    fn DispatchMessageW(msg: *const Message) -> isize;
    fn BeginPaint(hwnd: Handle, paint: *mut Paint) -> Handle;
    fn EndPaint(hwnd: Handle, paint: *const Paint) -> i32;
    fn FillRect(dc: Handle, rect: *const Rect, brush: Handle) -> i32;
    fn DrawTextW(dc: Handle, text: *const u16, len: i32, rect: *mut Rect, format: u32) -> i32;
    fn MessageBoxW(hwnd: Handle, text: *const u16, caption: *const u16, kind: u32) -> i32;
    fn SetTimer(hwnd: Handle, id: usize, interval: u32, callback: Handle) -> usize;
    fn DestroyWindow(hwnd: Handle) -> i32;
    fn PostQuitMessage(code: i32);
    fn SetProcessDPIAware() -> i32;
    fn RegisterWindowMessageW(name: *const u16) -> u32;
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateSolidBrush(color: u32) -> Handle;
    fn DeleteObject(object: Handle) -> i32;
    fn SetTextColor(dc: Handle, color: u32) -> u32;
    fn SetBkMode(dc: Handle, mode: i32) -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
unsafe extern "system" fn window_proc(hwnd: Handle, msg: u32, wp: usize, lp: isize) -> isize {
    match msg {
        0x000f => {
            unsafe {
                let mut paint: Paint = std::mem::zeroed();
                let dc = BeginPaint(hwnd, &mut paint);
                let brush = CreateSolidBrush(0x202020);
                FillRect(dc, &paint.rect, brush);
                DeleteObject(brush);
                SetTextColor(dc, 0xf0f0f0);
                SetBkMode(dc, 1);
                let text = wide("PROBE · CPU 12%  GPU 8%  RAM 42%\n↓ 1.2 MB/s   ↑ 240 KB/s");
                let mut rect = Rect {
                    left: 8,
                    top: 4,
                    right: 385,
                    bottom: 55,
                };
                DrawTextW(dc, text.as_ptr(), -1, &mut rect, 0x10);
                EndPaint(hwnd, &paint);
            }
            0
        }
        0x0202 => {
            unsafe {
                MessageBoxW(hwnd,wide("Taskbar placement probe only.\nAll displayed numbers are fixed sample values.\nThis is not the monitoring application.").as_ptr(),wide("Taskbar integration experiment").as_ptr(),0);
            }
            0
        }
        0x0113 => {
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        0x0002 => {
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        0x001a | 0x007e => {
            refresh_placement(hwnd);
            0
        }
        _ => {
            if msg == unsafe { RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()) } {
                refresh_placement(hwnd);
                return 0;
            }
            unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
        }
    }
}
fn refresh_placement(hwnd: Handle) {
    unsafe {
        let bar = FindWindowW(wide("Shell_TrayWnd").as_ptr(), std::ptr::null());
        let mut r = Rect::default();
        if bar.is_null() || GetWindowRect(bar, &mut r) == 0 {
            ShowWindow(hwnd, 0);
            return;
        }
        let mut desktop = Rect::default();
        let screen = FindWindowW(wide("Progman").as_ptr(), std::ptr::null());
        if !screen.is_null()
            && GetWindowRect(screen, &mut desktop) != 0
            && r.bottom > desktop.bottom
        {
            ShowWindow(hwnd, 0);
            println!("HIDDEN: auto-hidden taskbar");
            return;
        }
        match inspect_controls(bar)
            .ok()
            .and_then(|regions| placement::find_gap(r.left, r.right, 390, &regions))
        {
            Some(x) => {
                SetWindowPos(
                    hwnd,
                    -1isize as Handle,
                    x,
                    r.top,
                    390,
                    r.bottom - r.top,
                    0x0010,
                );
                ShowWindow(hwnd, 4);
                println!("REPOSITIONED {x},{}", r.top);
            }
            None => {
                ShowWindow(hwnd, 0);
                println!("HIDDEN: geometry unavailable or taskbar full");
            }
        }
    }
}
mod placement;
fn inspect_controls(taskbar: Handle) -> windows::core::Result<Vec<(i32, i32)>> {
    use windows::Win32::{Foundation::HWND, System::Com::*, UI::Accessibility::*};
    let mut occupied = Vec::new();
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
        let root = automation.ElementFromHandle(HWND(taskbar))?;
        let condition = automation.CreateTrueCondition()?;
        let elements = root.FindAll(TreeScope_Descendants, &condition)?;
        for index in 0..elements.Length()? {
            let element = elements.GetElement(index)?;
            if element.CurrentControlType()? == UIA_ButtonControlTypeId {
                let bounds = element.CurrentBoundingRectangle()?;
                if bounds.right > bounds.left {
                    occupied.push((bounds.left, bounds.right));
                }
                println!(
                    "CONTROL {:?}: {},{} to {},{}",
                    element.CurrentName()?,
                    bounds.left,
                    bounds.top,
                    bounds.right,
                    bounds.bottom
                );
            }
        }
        drop(elements);
        drop(root);
        drop(automation);
        CoUninitialize();
    }
    Ok(occupied)
}
fn main() {
    unsafe {
        SetProcessDPIAware();
        let taskbar = FindWindowW(wide("Shell_TrayWnd").as_ptr(), std::ptr::null());
        let mut rect = Rect::default();
        if taskbar.is_null() || GetWindowRect(taskbar, &mut rect) == 0 {
            eprintln!("No accessible Explorer taskbar");
            std::process::exit(1);
        }
        println!(
            "Taskbar: {},{} to {},{}",
            rect.left, rect.top, rect.right, rect.bottom
        );
        let mut child = null_mut();
        loop {
            child = FindWindowExW(taskbar, child, std::ptr::null(), std::ptr::null());
            if child.is_null() {
                break;
            }
            let mut r = Rect::default();
            GetWindowRect(child, &mut r);
            println!(
                "Child region: {},{} to {},{}",
                r.left, r.top, r.right, r.bottom
            );
        }
        let width = 390;
        let height = rect.bottom - rect.top;
        let occupied = match inspect_controls(taskbar) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("Semantic geometry unavailable: {error}");
                std::process::exit(4);
            }
        };
        let Some(x) = placement::find_gap(rect.left, rect.right, width, &occupied) else {
            println!("HIDDEN: insufficient safe taskbar space");
            return;
        };
        println!("Safe initial placement: x={x}, width={width}");
        if std::env::args().any(|a| a == "--inspect") {
            return;
        }
        let instance = GetModuleHandleW(std::ptr::null());
        let name = wide("TaskbarMonitorFeasibilityProbe");
        let class = Class {
            style: 0,
            proc: Some(window_proc),
            cls_extra: 0,
            wnd_extra: 0,
            instance,
            icon: null_mut(),
            cursor: null_mut(),
            background: null_mut(),
            menu: std::ptr::null(),
            name: name.as_ptr(),
        };
        if RegisterClassW(&class) == 0 {
            std::process::exit(2);
        }
        let hwnd = CreateWindowExW(
            0x08000080,
            name.as_ptr(),
            wide("Taskbar integration probe (sample data)").as_ptr(),
            0x80000000,
            x,
            rect.top,
            width,
            height,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        );
        if hwnd.is_null() {
            std::process::exit(3);
        }
        SetWindowPos(hwnd, -1isize as Handle, x, rect.top, width, height, 0x0010);
        ShowWindow(hwnd, 4);
        SetTimer(hwnd, 1, 120_000, null_mut());
        let mut message: Message = std::mem::zeroed();
        println!(
            "Probe running for at most 120 seconds; click for sample-data disclosure. Class bytes: {}",
            size_of::<Class>()
        );
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}
