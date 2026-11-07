#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod displays;
mod history;
mod layout;
mod metrics;
#[path = "../tools/placement.rs"]
mod placement;
mod sampler;
mod scan;
mod settings;
mod worker;
use std::{ffi::c_void, ptr::null_mut, time::Instant};
use windows::{
    Win32::{System::Com::*},
    core::w,
};
type H = *mut c_void;
#[repr(C)]
#[derive(Default, Copy, Clone)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct Class {
    style: u32,
    proc: Option<unsafe extern "system" fn(H, u32, usize, isize) -> isize>,
    a: i32,
    b: i32,
    instance: H,
    icon: H,
    cursor: H,
    background: H,
    menu: *const u16,
    name: *const u16,
}
#[repr(C)]
struct Message {
    hwnd: H,
    msg: u32,
    wp: usize,
    lp: isize,
    time: u32,
    x: i32,
    y: i32,
    private: u32,
}
#[repr(C)]
struct Paint {
    dc: H,
    erase: i32,
    rect: Rect,
    restore: i32,
    update: i32,
    reserved: [u8; 32],
}
#[repr(C)]
struct Notify {
    size: u32,
    hwnd: H,
    id: u32,
    flags: u32,
    callback: u32,
    icon: H,
    tip: [u16; 128],
    state: u32,
    mask: u32,
    info: [u16; 256],
    version: u32,
    title: [u16; 64],
    info_flags: u32,
    guid: [u8; 16],
    balloon: H,
}
#[link(name = "user32")]
unsafe extern "system" {
    fn FindWindowW(class: *const u16, title: *const u16) -> H;
    fn IsChild(parent: H, child: H) -> i32;
    fn GetWindowRect(hwnd: H, r: *mut Rect) -> i32;
    fn GetFocus() -> H;
    fn RegisterClassW(c: *const Class) -> u16;
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: H,
        menu: H,
        instance: H,
        param: H,
    ) -> H;
    fn DefWindowProcW(hwnd: H, msg: u32, wp: usize, lp: isize) -> isize;
    fn ShowWindow(hwnd: H, cmd: i32) -> i32;
    fn SetWindowPos(hwnd: H, after: H, x: i32, y: i32, width: i32, height: i32, flags: u32) -> i32;
    fn GetMessageW(m: *mut Message, hwnd: H, min: u32, max: u32) -> i32;
    fn TranslateMessage(m: *const Message) -> i32;
    fn DispatchMessageW(m: *const Message) -> isize;
    fn IsDialogMessageW(hwnd: H, m: *mut Message) -> i32;
    fn BeginPaint(hwnd: H, p: *mut Paint) -> H;
    fn EndPaint(hwnd: H, p: *const Paint) -> i32;
    fn FillRect(dc: H, r: *const Rect, brush: H) -> i32;
    fn DrawTextW(dc: H, text: *const u16, len: i32, r: *mut Rect, flags: u32) -> i32;
    fn GetDC(hwnd: H) -> H;
    fn ReleaseDC(hwnd: H, dc: H) -> i32;
    fn GetSystemMetrics(index: i32) -> i32;
    fn GetClientRect(hwnd: H, rect: *mut Rect) -> i32;
    fn ScrollWindowEx(hwnd: H, dx: i32, dy: i32, scroll: *const Rect, clip: *const Rect, update: H, rect: *mut Rect, flags: u32) -> i32;
    fn SetScrollRange(hwnd: H, bar: i32, min: i32, max: i32, redraw: i32) -> i32;
    fn SetScrollPos(hwnd: H, bar: i32, pos: i32, redraw: i32) -> i32;
    fn GetScrollPos(hwnd: H, bar: i32) -> i32;
    fn SetTimer(hwnd: H, id: usize, ms: u32, callback: H) -> usize;
    fn KillTimer(hwnd: H, id: usize) -> i32;
    fn PostQuitMessage(code: i32);
    fn DestroyWindow(hwnd: H) -> i32;
    fn SetProcessDPIAware() -> i32;
    fn RegisterWindowMessageW(name: *const u16) -> u32;
    fn InvalidateRect(hwnd: H, r: *const Rect, erase: i32) -> i32;
    fn SetWindowTextW(hwnd: H, text: *const u16) -> i32;
    fn GetWindowTextW(hwnd: H, text: *mut u16, max: i32) -> i32;
    fn SendMessageW(hwnd: H, msg: u32, wp: usize, lp: isize) -> isize;
    fn GetWindowLongPtrW(hwnd: H, index: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: H, index: i32, value: isize) -> isize;
    fn SetLayeredWindowAttributes(hwnd: H, key: u32, alpha: u8, flags: u32) -> i32;
    fn LoadIconW(instance: H, name: *const u16) -> H;
    fn LoadCursorW(instance: H, name: *const u16) -> H;
    fn CreatePopupMenu() -> H;
    fn AppendMenuW(menu: H, flags: u32, id: usize, label: *const u16) -> i32;
    fn TrackPopupMenu(
        menu: H,
        flags: u32,
        x: i32,
        y: i32,
        reserved: i32,
        hwnd: H,
        r: *const Rect,
    ) -> i32;
    fn DestroyMenu(menu: H) -> i32;
    fn GetCursorPos(p: *mut [i32; 2]) -> i32;
    fn SetForegroundWindow(hwnd: H) -> i32;
    fn MessageBoxW(hwnd: H, text: *const u16, title: *const u16, flags: u32) -> i32;
    fn SetWinEventHook(
        min: u32,
        max: u32,
        module: H,
        callback: Option<unsafe extern "system" fn(H, u32, H, i32, i32, u32, u32)>,
        process: u32,
        thread: u32,
        flags: u32,
    ) -> H;
    fn UnhookWinEvent(hook: H) -> i32;
    fn PostMessageW(hwnd: H, msg: u32, wp: usize, lp: isize) -> i32;
    fn SetParent(child: H, parent: H) -> H;
    fn GetParent(hwnd: H) -> H;
    fn ScreenToClient(hwnd: H, point: *mut [i32; 2]) -> i32;
}
#[link(name = "imm32")]
unsafe extern "system" {
    fn ImmAssociateContextEx(hwnd: H, context: H, flags: u32) -> i32;
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateSolidBrush(color: u32) -> H;
    fn DeleteObject(h: H) -> i32;
    fn CreatePen(style: i32, width: i32, color: u32) -> H;
    fn MoveToEx(dc: H, x: i32, y: i32, old: *mut [i32; 2]) -> i32;
    fn LineTo(dc: H, x: i32, y: i32) -> i32;
    fn EnumFontFamiliesExW(dc: H, font: *const LogFont, callback: Option<unsafe extern "system" fn(*const LogFont, *const c_void, u32, isize) -> i32>, data: isize, flags: u32) -> i32;
    fn SetTextColor(dc: H, color: u32) -> u32;
    fn SetBkMode(dc: H, mode: i32) -> i32;
    fn CreateFontW(
        height: i32,
        width: i32,
        escape: i32,
        orient: i32,
        weight: i32,
        italic: u32,
        under: u32,
        strike: u32,
        charset: u32,
        out: u32,
        clip: u32,
        quality: u32,
        pitch: u32,
        face: *const u16,
    ) -> H;
    fn SelectObject(dc: H, h: H) -> H;
    fn GetTextExtentPoint32W(dc: H, text: *const u16, len: i32, size: *mut [i32; 2]) -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> H;
    fn CreateMutexW(attributes: H, owner: i32, name: *const u16) -> H;
    fn GetLastError() -> u32;
    fn CloseHandle(h: H) -> i32;
}
#[link(name = "shell32")]
unsafe extern "system" {
    fn Shell_NotifyIconW(message: u32, data: *const Notify) -> i32;
}
#[repr(C)]
struct LogFont {
    height: i32, width: i32, escapement: i32, orientation: i32, weight: i32,
    italic: u8, underline: u8, strikeout: u8, charset: u8,
    out_precision: u8, clip_precision: u8, quality: u8, pitch_family: u8,
    face: [u16; 32],
}

unsafe extern "system" fn collect_font(font: *const LogFont, _: *const c_void, _: u32, data: isize) -> i32 {
    unsafe {
        let face = &(*font).face;
        let len = face.iter().position(|&c| c == 0).unwrap_or(face.len());
        let name = String::from_utf16_lossy(&face[..len]);
        if !name.is_empty() && !name.starts_with('@') {
            (&mut *(data as *mut Vec<String>)).push(name);
        }
        1
    }
}

fn installed_fonts(hwnd: H) -> Vec<String> {
    unsafe {
        let dc = GetDC(hwnd);
        let mut fonts: Vec<String> = Vec::new();
        if !dc.is_null() {
            let mut font: LogFont = std::mem::zeroed();
            font.charset = 1;
            EnumFontFamiliesExW(dc, &font, Some(collect_font), &mut fonts as *mut _ as isize, 0);
            ReleaseDC(hwnd, dc);
        }
        fonts.sort_unstable_by_key(|name| name.to_lowercase());
        fonts.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        fonts
    }
}

fn draw_history(dc: H, history: &history::History, index: usize, rect: Rect, color: u32) {
    let count = history.points(index).count();
    if count == 0 || rect.right <= rect.left || rect.bottom <= rect.top {
        return;
    }
    let max = if index < 3 { 100.0 } else {
        history.points(index).flatten().fold(0.0_f64, f64::max).max(1.0)
    };
    unsafe {
        let pen = CreatePen(0, 1, color);
        if pen.is_null() { return; }
        let old = SelectObject(dc, pen);
        let mut connected = false;
        for (i, point) in history.points(index).enumerate() {
            if let Some(value) = point.filter(|v| v.is_finite()) {
                let x = rect.right - 1 - ((count - 1 - i) as i32 * (rect.right - rect.left - 1) / 59);
                let y = rect.bottom - 1 - (value.clamp(0.0, max) / max * (rect.bottom - rect.top - 1) as f64) as i32;
                if connected { LineTo(dc, x, y); } else { MoveToEx(dc, x, y, null_mut()); }
                connected = true;
            } else {
                connected = false;
            }
        }
        SelectObject(dc, old);
        DeleteObject(pen);
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn set_startup(enabled: bool) -> Result<(), String> {
    use windows::Win32::System::Registry::*;
    unsafe {
        let mut key = HKEY::default();
        RegCreateKeyW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            &mut key,
        )
        .ok()
        .map_err(|e| e.to_string())?;
        let result = if enabled {
            let executable = std::env::current_exe().map_err(|e| e.to_string())?;
            let value = wide(&format!("\"{}\"", executable.display()));
            RegSetValueExW(
                key,
                w!("TaskbarSystemMonitor"),
                Some(0),
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    value.as_ptr().cast(),
                    value.len() * 2,
                )),
            )
            .ok()
        } else {
            let status = RegDeleteValueW(key, w!("TaskbarSystemMonitor"));
            if status.0 == 2 { Ok(()) } else { status.ok() }
        };
        RegCloseKey(key).ok().ok();
        result.map_err(|e| e.to_string())
    }
}
unsafe extern "system" fn shell_changed(
    _hook: H,
    _event: u32,
    target: H,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    unsafe {
        let meter = FindWindowW(w!("TaskbarSystemMonitor").as_ptr(), std::ptr::null());
        if meter.is_null() {
            return;
        }
        let taskbar_event = displays::list()
            .iter()
            .any(|d| target == d.taskbar || IsChild(d.taskbar, target) != 0);
        if layout::relevant_shell_event(_event, taskbar_event) {
            PostMessageW(meter, if _event == 3 { 0x8003 } else { 0x8002 }, 0, 0);
        }
    }
}
struct App {
    config: settings::Settings,
    worker: worker::Worker,
    snapshot: sampler::Snapshot,
    history: history::History,
    history_stamp: Option<Instant>,
    devices: Vec<sampler::Adapter>,
    gpus: Vec<(String, String)>,
    font: H,
    small: H,
    details: H,
    settings: H,
    controller: H,
    meter: H,
    parent: H,
    rejected: H,
    retries: u8,
    scan: scan::Scanner,
    geometry: Option<scan::Layout>,
    scan_tries: u8,
    stale: bool,
    meter_height: i32,
    visible: bool,
    blocked: bool,
    last: Instant,
    notice: String,
    hook: H,
    foreground_hook: H,
}
unsafe fn app(hwnd: H) -> &'static mut App {
    unsafe { &mut *(GetWindowLongPtrW(hwnd, -21) as *mut App) }
}
fn install_shell_hook() -> H {
    unsafe {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn GetWindowThreadProcessId(hwnd: H, process: *mut u32) -> u32;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(
            FindWindowW(w!("Shell_TrayWnd").as_ptr(), std::ptr::null()),
            &mut pid,
        );
        if pid == 0 {
            return null_mut();
        }
        SetWinEventHook(0x8000, 0x800c, null_mut(), Some(shell_changed), pid, 0, 0)
    }
}
fn is_light(theme: u32) -> bool {
    if theme != 0 {
        return theme == 1;
    }
    use windows::Win32::System::Registry::*;
    unsafe {
        let mut value = 0u32;
        let mut bytes = 4;
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut bytes),
        )
        .is_ok()
            && value != 0
    }
}
fn fluent(hwnd: H, light: bool) {
    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmSetWindowAttribute(hwnd: H, attribute: u32, value: *const c_void, size: u32) -> i32;
    }
    unsafe {
        // Unknown attributes are ignored by older builds, so no version check is needed.
        let dark = (!light) as i32;
        DwmSetWindowAttribute(hwnd, 20, &dark as *const i32 as *const c_void, 4);
        let round = 2i32;
        DwmSetWindowAttribute(hwnd, 33, &round as *const i32 as *const c_void, 4);
    }
}
fn rgb(r: u32, g: u32, b: u32) -> u32 {
    r | (g << 8) | (b << 16)
}
fn meter_transparency(hwnd: H, enabled: bool, theme: u32) {
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, -20) as u32;
        if enabled {
            SetWindowLongPtrW(hwnd, -20, (ex | 0x80000) as isize);
            let key = if is_light(theme) {
                0x00f5f5f5
            } else {
                0x00202020
            };
            SetLayeredWindowAttributes(hwnd, key, 255, 1);
        } else {
            SetWindowLongPtrW(hwnd, -20, (ex & !0x80000) as isize);
        }
        SetWindowPos(hwnd, null_mut(), 0, 0, 0, 0, 0x27);
    }
}
fn percentage(v: Option<f64>) -> String {
    v.map(|v| format!("{v:.0}%")).unwrap_or_else(|| "--".into())
}
fn rate(v: Option<f64>, bits: bool) -> String {
    let Some(mut v) = v else { return "--".into() };
    if bits {
        v *= 8.0;
    }
    let units = if bits {
        ["bps", "Kbps", "Mbps", "Gbps"]
    } else {
        ["B/s", "KB/s", "MB/s", "GB/s"]
    };
    let mut unit = 0;
    while v >= 1000.0 && unit < 3 {
        v /= 1000.0;
        unit += 1;
    }
    format!("{v:.1} {}", units[unit])
}
#[repr(C)]
struct DrawItem {
    kind: u32,
    id: u32,
    item: u32,
    action: u32,
    state: u32,
    window: H,
    dc: H,
    rect: Rect,
    data: usize,
}
fn draw_button(item: &DrawItem, light: bool) {
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn CreatePen(style: i32, width: i32, color: u32) -> H;
        fn RoundRect(dc: H, left: i32, top: i32, right: i32, bottom: i32, w: i32, h: i32) -> i32;
    }
    unsafe {
        let primary = item.id == 201;
        let pressed = item.state & 1 != 0;
        let (background, border, text) = if primary {
            if light {
                (
                    rgb(0x0f, 0x6c, 0xbd),
                    rgb(0x0f, 0x6c, 0xbd),
                    rgb(0xff, 0xff, 0xff),
                )
            } else {
                (
                    rgb(0x4c, 0xc2, 0xff),
                    rgb(0x4c, 0xc2, 0xff),
                    rgb(0x0a, 0x0a, 0x0a),
                )
            }
        } else if light {
            (0xffffff, 0xd0d0d0, 0x202020)
        } else {
            (0x2d2d2d, 0x3d3d3d, 0xf4f4f4)
        };
        let background = if pressed {
            (background & 0xfefefe) >> 1
        } else {
            background
        };
        let brush = CreateSolidBrush(background);
        let pen = CreatePen(0, 1, border);
        let old_brush = SelectObject(item.dc, brush);
        let old_pen = SelectObject(item.dc, pen);
        let r = item.rect;
        RoundRect(item.dc, r.left, r.top, r.right, r.bottom, 8, 8);
        SelectObject(item.dc, old_brush);
        SelectObject(item.dc, old_pen);
        DeleteObject(brush);
        DeleteObject(pen);
        let mut label = [0u16; 128];
        let count = GetWindowTextW(item.window, label.as_mut_ptr(), 128);
        SetBkMode(item.dc, 1);
        SetTextColor(item.dc, text);
        let mut area = Rect {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        };
        DrawTextW(
            item.dc,
            label.as_ptr(),
            count,
            &mut area,
            0x1 | 0x4 | 0x20 | 0x800,
        );
    }
}
fn control(
    parent: H,
    class: &str,
    text: &str,
    id: usize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    style: u32,
) -> H {
    unsafe {
        let h = CreateWindowExW(
            0,
            &wide(class)[0],
            wide(text).as_ptr(),
            0x50000000 | style,
            x,
            y,
            width,
            height,
            parent,
            id as H,
            GetModuleHandleW(std::ptr::null()),
            null_mut(),
        );
        let a = app(parent);
        if !is_light(a.config.theme) && matches!(class, "COMBOBOX" | "EDIT") {
            #[link(name = "uxtheme")]
            unsafe extern "system" {
                fn SetWindowTheme(hwnd: H, app: *const u16, id: *const u16) -> i32;
            }
            SetWindowTheme(h, w!("DarkMode_Explorer").as_ptr(), std::ptr::null());
        }
        if class == "BUTTON" && matches!(style & 0xf, 3 | 4 | 9) {
            #[link(name = "uxtheme")]
            unsafe extern "system" {
                fn SetWindowTheme(hwnd: H, app: *const u16, id: *const u16) -> i32;
            }
            SetWindowTheme(h, w!("").as_ptr(), w!("").as_ptr());
        }
        SendMessageW(h, 0x30, a.small as usize, 1);
        h
    }
}
fn notify(hwnd: H, remove: bool) {
    unsafe {
        let mut n: Notify = std::mem::zeroed();
        n.size = std::mem::size_of::<Notify>() as u32;
        n.hwnd = hwnd;
        n.id = 1;
        n.flags = 1 | 2 | 4;
        n.callback = 0x8001;
        n.icon = LoadIconW(GetModuleHandleW(std::ptr::null()), 101usize as *const u16);
        let a = app(hwnd);
        let tip = if a.blocked {
            "Kurmon: insufficient taskbar space. Click for details."
        } else {
            "Kurmon"
        };
        for (i, c) in tip.encode_utf16().enumerate() {
            n.tip[i] = c;
        }
        Shell_NotifyIconW(if remove { 2 } else { 0 }, &n);
    }
}
fn fullscreen(screen: displays::Rect) -> bool {
    unsafe {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn GetForegroundWindow() -> H;
            fn GetClassNameW(hwnd: H, name: *mut u16, max: i32) -> i32;
        }
        let foreground = GetForegroundWindow();
        let mut class = [0u16; 100];
        let length = GetClassNameW(foreground, class.as_mut_ptr(), 100);
        let name = String::from_utf16_lossy(&class[..length as usize]);
        if ["Progman", "WorkerW", "Shell_TrayWnd"].contains(&name.as_str()) {
            return false;
        }
        let mut r = Rect::default();
        GetWindowRect(foreground, &mut r) != 0
            && r.left <= screen.left
            && r.top <= screen.top
            && r.right >= screen.right
            && r.bottom >= screen.bottom
    }
}
unsafe fn create_meter(a: &mut App, bar: H) -> H {
    unsafe {
        let meter = CreateWindowExW(
            0x08000080,
            w!("KurmonMeter").as_ptr(),
            w!("Kurmon").as_ptr(),
            0x80000000 | 0x00040000,
            0,
            0,
            a.config.width,
            60,
            null_mut(),
            null_mut(),
            GetModuleHandleW(std::ptr::null()),
            null_mut(),
        );
        if meter.is_null() {
            return meter;
        }
        SetWindowLongPtrW(meter, -21, a as *mut App as isize);
        // A non-text child of Explorer's input tree must not own an IME context.
        ImmAssociateContextEx(meter, null_mut(), 0);
        meter_transparency(meter, a.config.transparent, a.config.theme);
        let style = GetWindowLongPtrW(meter, -16) as u32;
        SetWindowLongPtrW(meter, -16, ((style & !0x80000000) | 0x40000000) as isize);
        SetParent(meter, bar);
        if GetParent(meter) == bar {
            a.parent = bar;
            a.retries = 0;
            KillTimer(a.controller, 4);
        } else {
            // Builds that refuse cross-process parenting keep the plain overlay.
            let style = GetWindowLongPtrW(meter, -16) as u32;
            SetWindowLongPtrW(meter, -16, ((style & !0x40000000) | 0x80000000) as isize);
            a.parent = null_mut();
            a.rejected = bar;
            a.retries = a.retries.saturating_add(1);
            if a.retries <= 6 {
                // Shell refusals are usually transient (busy taskbar thread).
                SetTimer(a.controller, 4, 5000, null_mut());
            }
        }
        meter
    }
}
unsafe fn place(a: &mut App) {
    unsafe {
        if let Some(display) = displays::selected(&a.config.display) {
            if !fullscreen(display.screen) {
                let bar = display.taskbar;
                let mut r = Rect::default();
                if bar.is_null() || GetWindowRect(bar, &mut r) == 0 {
                    if !a.meter.is_null() {
                        ShowWindow(a.meter, 0);
                    }
                    return;
                }
                // A failed attach while the shell is mid-animation is retried on the next
                // shell event, not on every placement tick.
                let stale = a.stale;
                if stale {
                    a.stale = false;
                    a.rejected = null_mut();
                    a.retries = 0;
                    a.scan_tries = 0;
                }
                let attached = GetParent(a.meter);
                let refused = !a.meter.is_null() && a.rejected == bar && attached != bar;
                if !refused
                    && layout::replace_meter(attached as usize, bar as usize, !a.meter.is_null())
                {
                    if !a.meter.is_null() {
                        DestroyWindow(a.meter);
                        a.meter = null_mut();
                    }
                    let created = create_meter(a, bar);
                    a.meter = created;
                    if created.is_null() {
                        return;
                    }
                }
                let meter = a.meter;
                if stale || a.geometry.as_ref().map_or(true, |g| g.bar != bar as usize) {
                    if a.scan_tries < 15 {
                        a.scan_tries += 1;
                        a.scan.request(bar);
                        SetTimer(a.controller, 3, 200, null_mut());
                    }
                }
                if let Some(layout) = a.scan.latest() {
                    if layout.bar == bar as usize {
                        a.scan_tries = 0;
                    }
                    a.geometry = Some(layout);
                }
                let Some(geometry) = a.geometry.as_ref().filter(|g| g.bar == bar as usize) else {
                    // Wait for the off-thread scan instead of covering app buttons.
                    ShowWindow(meter, 0);
                    return;
                };
                let Some((x, y, height)) = layout::meter_position(
                    (r.left, r.top, r.right, r.bottom),
                    a.config.width,
                    a.config.position,
                    a.config.offset,
                    &geometry.occupied,
                ) else {
                    a.blocked = true;
                    ShowWindow(meter, 0);
                    return;
                };
                a.blocked = false;
                a.meter_height = height;
                KillTimer(a.controller, 3);
                let (x, y, after) = if GetParent(meter) == bar {
                    let mut point = [x, y];
                    ScreenToClient(bar, &mut point);
                    (point[0], point[1], null_mut())
                } else {
                    (x, y, -1isize as H)
                };
                SetWindowPos(meter, after, x, y, a.config.width, height, 0x10);
                ShowWindow(meter, if a.visible { 4 } else { 0 });
                return;
            }
        }
        if !a.meter.is_null() {
            ShowWindow(a.meter, 0);
        }
    }
}
fn position_details(a: &App, panel: H) {
    unsafe {
        if let Some(display) = displays::selected(&a.config.display) {
            let mut meter = Rect::default();
            let mut rect = Rect::default();
            if GetWindowRect(a.meter, &mut meter) != 0 && GetWindowRect(panel, &mut rect) != 0 {
                let screen = display.screen;
                let (x, y) = layout::popup_position(
                    (meter.left, meter.top, meter.right, meter.bottom),
                    (rect.right - rect.left, rect.bottom - rect.top),
                    (screen.left, screen.top, screen.right, screen.bottom),
                );
                SetWindowPos(panel, null_mut(), x, y, 0, 0, 0x1 | 0x4 | 0x10);
            }
        }
    }
}
fn show_menu(hwnd: H) {
    unsafe {
        let menu = CreatePopupMenu();
        for (id, label) in [
            (1, "Open details"),
            (2, "Settings"),
            (3, "Show / hide meter"),
            (4, "Exit"),
        ] {
            AppendMenuW(menu, 0, id, wide(label).as_ptr());
        }
        let mut p = [0, 0];
        GetCursorPos(&mut p);
        SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(menu, 0x100, p[0], p[1], 0, hwnd, std::ptr::null());
        DestroyMenu(menu);
        match cmd {
            1 => details(hwnd),
            2 => open_settings(hwnd),
            3 => {
                app(hwnd).visible = !app(hwnd).visible;
                place(app(hwnd));
            }
            4 => { DestroyWindow(hwnd); }
            _ => {}
        }
    }
}
fn details(hwnd: H) {
    unsafe {
        let a = app(hwnd);
        if !a.details.is_null() {
            position_details(a, a.details);
            ShowWindow(a.details, 5);
            SetForegroundWindow(a.details);
            return;
        }
        let h = CreateWindowExW(
            0x80,
            wide("MonitorPanel").as_ptr(),
            wide("Kurmon overview").as_ptr(),
            0x80c80000,
            100,
            500,
            450,
            570,
            a.controller,
            null_mut(),
            GetModuleHandleW(std::ptr::null()),
            null_mut(),
        );
        SetWindowLongPtrW(h, -21, a as *mut App as isize);
        a.details = h;
        fluent(h, is_light(a.config.theme));
        control(h, "STATIC", "Collecting readings", 600, 24, 20, 390, 240, 0);
        control(
            h,
            "BUTTON",
            "Settings",
            501,
            24,
            480,
            110,
            32,
            0x10000 | 0xb,
        );
        control(
            h,
            "BUTTON",
            "Task Manager",
            502,
            146,
            480,
            130,
            32,
            0x10000 | 0xb,
        );
        position_details(a, h);
        ShowWindow(h, 5);
        SetForegroundWindow(h);
    }
}
fn open_settings(hwnd: H) {
    unsafe {
        let a = app(hwnd);
        if !a.settings.is_null() {
            ShowWindow(a.settings, 5);
            return;
        }
        let h = CreateWindowExW(
            0x80,
            wide("MonitorPanel").as_ptr(),
            wide("Kurmon Settings").as_ptr(),
            0x80e80000,
            20,
            20,
            520,
            (GetSystemMetrics(1) - 40).min(930),
            a.controller,
            null_mut(),
            GetModuleHandleW(std::ptr::null()),
            null_mut(),
        );
        SetWindowLongPtrW(h, -21, a as *mut App as isize);
        a.settings = h;
        fluent(h, is_light(a.config.theme));
        control(h, "STATIC", "Network adapter", 0, 24, 20, 440, 22, 0);
        let combo = control(
            h,
            "COMBOBOX",
            "",
            101,
            24,
            45,
            450,
            280,
            0x10000 | 3 | 0x200000,
        );
        a.devices = sampler::adapters();
        for (i, d) in a.devices.iter().enumerate() {
            SendMessageW(combo, 0x143, 0, wide(&d.name).as_ptr() as isize);
            if d.id == a.config.adapter {
                SendMessageW(combo, 0x14e, i, 0);
            }
        }
        control(h, "STATIC", "Refresh interval", 0, 24, 90, 200, 22, 0);
        let interval = control(h, "COMBOBOX", "", 102, 24, 115, 200, 140, 0x10000 | 3);
        for label in ["1 second", "2 seconds", "5 seconds"] {
            SendMessageW(interval, 0x143, 0, wide(label).as_ptr() as isize);
        }
        SendMessageW(
            interval,
            0x14e,
            match a.config.interval {
                2 => 1,
                5 => 2,
                _ => 0,
            },
            0,
        );
        control(h, "STATIC", "Appearance", 0, 260, 90, 200, 22, 0);
        let theme = control(h, "COMBOBOX", "", 103, 260, 115, 210, 140, 0x10000 | 3);
        for label in ["System", "Light", "Dark"] {
            SendMessageW(theme, 0x143, 0, wide(label).as_ptr() as isize);
        }
        SendMessageW(theme, 0x14e, a.config.theme as usize, 0);
        let bits = control(
            h,
            "BUTTON",
            "Show network speed in bits per second",
            104,
            24,
            165,
            440,
            26,
            0x10000 | 3,
        );
        SendMessageW(bits, 0xf1, a.config.bits as usize, 0);
        control(h, "STATIC", "Visible readings", 0, 24, 210, 440, 22, 0);
        for (i, label) in ["CPU", "GPU", "RAM", "Download", "Upload"]
            .iter()
            .enumerate()
        {
            let check = control(
                h,
                "BUTTON",
                label,
                140 + i,
                24 + (i as i32 % 3) * 145,
                240 + (i as i32 / 3) * 35,
                140,
                26,
                0x10000 | 3,
            );
            SendMessageW(check, 0xf1, a.config.visible[i] as usize, 0);
        }
        control(h, "STATIC", "GPU adapter", 0, 24, 310, 440, 22, 0);
        let gpu = control(h, "COMBOBOX", "", 105, 24, 335, 450, 180, 0x10000 | 3);
        for (i, (_id, name)) in a.gpus.iter().enumerate() {
            SendMessageW(gpu, 0x143, 0, wide(name).as_ptr() as isize);
            if name.eq_ignore_ascii_case(&a.config.gpu) {
                SendMessageW(gpu, 0x14e, i, 0);
            }
        }
        control(
            h,
            "STATIC",
            "Reading order (comma-separated labels)",
            0,
            24,
            375,
            450,
            22,
            0,
        );
        control(
            h,
            "EDIT",
            &a.config.order,
            106,
            24,
            400,
            450,
            26,
            0x10000 | 0x800000,
        );
        let startup = control(
            h,
            "BUTTON",
            "Start automatically when I sign in",
            107,
            24,
            438,
            450,
            26,
            0x10000 | 3,
        );
        SendMessageW(startup, 0xf1, a.config.startup as usize, 0);
        control(h, "STATIC", "Display", 0, 24, 470, 75, 22, 0);
        let selector = control(
            h,
            "COMBOBOX",
            "",
            108,
            100,
            466,
            374,
            150,
            0x10000 | 3 | 0x200000,
        );
        for (i, display) in displays::list().iter().enumerate() {
            SendMessageW(selector, 0x143, 0, wide(&display.id).as_ptr() as isize);
            if display.id == a.config.display || (a.config.display.is_empty() && i == 0) {
                SendMessageW(selector, 0x14e, i, 0);
            }
        }
        control(h, "STATIC", "Meter font", 0, 24, 515, 90, 22, 0);
        let fonts = installed_fonts(h);
        let font = control(h, "COMBOBOX", "", 120, 120, 510, 350, 250, 0x10000 | 3 | 0x200000);
        let mut selected = false;
        for name in &fonts {
            let index = SendMessageW(font, 0x143, 0, wide(name).as_ptr() as isize);
            if name.eq_ignore_ascii_case(&a.config.font) {
                SendMessageW(font, 0x14e, index as usize, 0);
                selected = true;
            }
        }
        if !selected && !a.config.font.is_empty() {
            let index = SendMessageW(font, 0x143, 0, wide(&a.config.font).as_ptr() as isize);
            SendMessageW(font, 0x14e, index as usize, 0);
        }
        control(h, "STATIC", "Position in taskbar", 0, 24, 550, 180, 22, 0);
        let position = control(h, "COMBOBOX", "", 109, 205, 545, 265, 150, 0x10000 | 3);
        for label in [
            "Automatic",
            "Left",
            "Center",
            "Right",
            "Custom (drag handle)",
        ] {
            SendMessageW(position, 0x143, 0, wide(label).as_ptr() as isize);
        }
        SendMessageW(position, 0x14e, (a.config.position + 1).max(0) as usize, 0);
        let lock = control(
            h,
            "BUTTON",
            "Lock position",
            112,
            24,
            585,
            220,
            26,
            0x10000 | 3,
        );
        SendMessageW(lock, 0xf1, a.config.locked as usize, 0);
        let transparent = control(
            h,
            "BUTTON",
            "Transparent background",
            114,
            24,
            620,
            240,
            26,
            0x10000 | 3,
        );
        SendMessageW(transparent, 0xf1, a.config.transparent as usize, 0);
        control(h, "STATIC", "Meter color", 0, 24, 655, 90, 22, 0);
        let palette = control(h, "COMBOBOX", "", 115, 120, 650, 250, 130, 0x10000 | 3);
        for label in ["Blue", "Green", "Amber"] {
            SendMessageW(palette, 0x143, 0, wide(label).as_ptr() as isize);
        }
        SendMessageW(palette, 0x14e, a.config.palette as usize, 0);
        control(h, "STATIC", "Show metric labels", 0, 24, 685, 440, 22, 0);
        for (i, label) in ["CPU", "GPU", "RAM", "Download", "Upload"].iter().enumerate() {
            let check = control(
                h, "BUTTON", label, 130 + i,
                24 + (i as i32 % 3) * 145, 710 + (i as i32 / 3) * 30,
                140, 26, 0x10000 | 3,
            );
            SendMessageW(check, 0xf1, a.config.labels[i] as usize, 0);
        }
        control(h, "STATIC", "RAM display", 0, 24, 780, 120, 22, 0);
        let ram_format = control(h, "COMBOBOX", "", 135, 145, 775, 235, 120, 0x10000 | 3);
        for label in ["Percent", "Used GiB", "Percent and used GiB"] {
            SendMessageW(ram_format, 0x143, 0, wide(label).as_ptr() as isize);
        }
        SendMessageW(ram_format, 0x14e, a.config.ram_format as usize, 0);
        control(h, "STATIC", "Show history charts", 0, 24, 815, 440, 22, 0);
        for (i, label) in ["CPU chart", "GPU chart", "RAM chart", "Download chart", "Upload chart"].iter().enumerate() {
            let check = control(h, "BUTTON", label, 150 + i,
                24 + (i as i32 % 3) * 145, 840 + (i as i32 / 3) * 30,
                140, 26, 0x10000 | 3);
            SendMessageW(check, 0xf1, a.config.charts[i] as usize, 0);
        }
        control(h, "BUTTON", "Save settings", 201, 24, 910, 160, 36, 0x10000 | 0xb);
        control(h, "BUTTON", "Cancel", 202, 200, 910, 110, 36, 0x10000 | 0xb);
        let mut client = Rect::default();
        GetClientRect(h, &mut client);
        SetScrollRange(h, 1, 0, (960 - client.bottom).max(0), 1);
        ShowWindow(h, 5);
        SetForegroundWindow(h);
    }
}
fn get_child(hwnd: H, id: i32) -> H {
    unsafe extern "system" {
        fn GetDlgItem(hwnd: H, id: i32) -> H;
    }
    unsafe { GetDlgItem(hwnd, id) }
}
unsafe fn scroll_settings(hwnd: H, target: i32) {
    unsafe {
        let mut client = Rect::default();
        GetClientRect(hwnd, &mut client);
        let old = GetScrollPos(hwnd, 1);
        let target = target.clamp(0, (960 - client.bottom).max(0));
        if target != old {
            SetScrollPos(hwnd, 1, target, 1);
            ScrollWindowEx(hwnd, 0, old - target, std::ptr::null(), std::ptr::null(), null_mut(), null_mut(), 3);
        }
    }
}
unsafe extern "system" fn proc(hwnd: H, msg: u32, wp: usize, lp: isize) -> isize {
    unsafe {
        if GetWindowLongPtrW(hwnd, -21) == 0 {
            return DefWindowProcW(hwnd, msg, wp, lp);
        }
        match msg {
            0x115 if hwnd == app(hwnd).settings => {
                let pos = GetScrollPos(hwnd, 1);
                let mut client = Rect::default();
                GetClientRect(hwnd, &mut client);
                let target = match wp & 0xffff {
                    0 => pos - 30, 1 => pos + 30,
                    2 => pos - client.bottom, 3 => pos + client.bottom,
                    4 | 5 => (wp >> 16) as i32,
                    6 => 0, 7 => 960, _ => pos,
                };
                scroll_settings(hwnd, target);
                0
            }
            0x20a if hwnd == app(hwnd).settings => {
                let delta = ((wp >> 16) as u16 as i16) as i32;
                scroll_settings(hwnd, GetScrollPos(hwnd, 1) - delta);
                0
            }
            0x83 if hwnd == app(hwnd).meter => 0,
            0x85 if hwnd == app(hwnd).meter => 0,
            0x84 if hwnd == app(hwnd).meter => {
                let a = app(hwnd);
                if !a.parent.is_null() {
                    // A child in Explorer's tree cannot start a caption move.
                    return 1;
                }
                let mut r = Rect::default();
                GetWindowRect(hwnd, &mut r);
                let x = (lp as u32 as u16 as i16) as i32;
                let y = ((lp as u32 >> 16) as u16 as i16) as i32;
                if a.config.locked {
                    if x >= r.right - 8 && y >= r.top && y < r.bottom {
                        11
                    } else {
                        1
                    }
                } else if x >= r.left && x < r.left + 8 && y >= r.top + 6 && y < r.bottom - 6 {
                    2
                } else {
                    DefWindowProcW(hwnd, msg, wp, lp)
                }
            }
            0x21 if hwnd == app(hwnd).meter => 3,
            0x232 if hwnd == app(hwnd).meter => {
                let a = app(hwnd);
                let mut r = Rect::default();
                GetWindowRect(hwnd, &mut r);
                a.config.width = (r.right - r.left).clamp(260, 1200);
                if let Some(display) = displays::selected(&a.config.display) {
                    let mut bar = Rect::default();
                    if GetWindowRect(display.taskbar, &mut bar) != 0 {
                        a.config.position = 3;
                        a.config.offset = (r.left - bar.left).max(0);
                    }
                }
                let _ = settings::save(&a.config);
                place(app(hwnd));
                InvalidateRect(app(hwnd).meter, std::ptr::null(), 0);
                0
            }
            0x8003 => {
                let a = app(hwnd);
                a.stale = true;
                place(a);
                0
            }
            0x8002 => {
                app(hwnd).stale = true;
                SetTimer(hwnd, 2, 200, null_mut());
                0
            }
            0x113 => {
                if wp == 2 {
                    KillTimer(hwnd, 2);
                    place(app(hwnd));
                    return 0;
                }
                if wp == 3 {
                    place(app(hwnd));
                    return 0;
                }
                if wp == 4 {
                    app(hwnd).rejected = null_mut();
                    place(app(hwnd));
                    return 0;
                }
                let a = app(hwnd);
                let (next, stamp, sampled) = match a.worker.latest() {
                    Some((snapshot, stamp)) => (snapshot, stamp, true),
                    None => {
                        if a.last.elapsed().as_secs() < u64::from(a.config.interval) * 3 {
                            return 0;
                        }
                        let mut stale = sampler::Snapshot::default();
                        stale.network_status = "Stale: waiting for sampler".into();
                        (stale, a.last, false)
                    }
                };
                let new_sample = sampled && a.history_stamp != Some(stamp);
                if new_sample {
                    a.history.push(&next);
                    a.history_stamp = Some(stamp);
                }
                let changed = percentage(next.cpu) != percentage(a.snapshot.cpu)
                    || percentage(next.gpu) != percentage(a.snapshot.gpu)
                    || percentage(next.ram) != percentage(a.snapshot.ram)
                    || (a.config.ram_format != 0 && format!("{:.1}", next.used as f64 / 1073741824.0) != format!("{:.1}", a.snapshot.used as f64 / 1073741824.0))
                    || rate(next.download, a.config.bits)
                        != rate(a.snapshot.download, a.config.bits)
                    || rate(next.upload, a.config.bits) != rate(a.snapshot.upload, a.config.bits);
                a.snapshot = next;
                a.last = stamp;
                if (changed || (new_sample && a.config.charts.iter().any(|&v| v))) && a.visible && !a.blocked && !a.meter.is_null() {
                    InvalidateRect(a.meter, std::ptr::null(), 0);
                }
                if !a.details.is_null() {
                    InvalidateRect(a.details, std::ptr::null(), 0);
                }
                0
            }
            0x205 if hwnd == app(hwnd).meter => {
                show_menu(hwnd);
                0
            }
            0x8001 => {
                if lp as u32 == 0x202 {
                    details(hwnd);
                } else if lp as u32 == 0x205 {
                    show_menu(hwnd);
                }
                0
            }
            0x202 if hwnd == app(hwnd).meter || hwnd == app(hwnd).controller => {
                details(hwnd);
                0
            }
            0x111 => {
                let id = wp & 0xffff;
                let a = app(hwnd);
                match id {
                    501 => open_settings(a.controller),
                    502 => {
                        let _ = std::process::Command::new("taskmgr.exe").spawn();
                    }
                    202 => {
                        DestroyWindow(hwnd);
                    }
                    201 => {
                        let mut next = a.config.clone();
                        let index = SendMessageW(get_child(hwnd, 101), 0x147, 0, 0);
                        if index >= 0 {
                            next.adapter = a.devices[index as usize].id;
                        }
                        let gpu = SendMessageW(get_child(hwnd, 105), 0x147, 0, 0);
                        if gpu >= 0 {
                            next.gpu = a.gpus[gpu as usize].1.clone();
                        }
                        let interval = SendMessageW(get_child(hwnd, 102), 0x147, 0, 0);
                        next.interval = match interval {
                            1 => 2,
                            2 => 5,
                            _ => 1,
                        };
                        next.theme = SendMessageW(get_child(hwnd, 103), 0x147, 0, 0).max(0) as u32;
                        next.bits = SendMessageW(get_child(hwnd, 104), 0xf0, 0, 0) == 1;
                        for i in 0..5 {
                            next.visible[i] =
                                SendMessageW(get_child(hwnd, 140 + i as i32), 0xf0, 0, 0) == 1;
                            next.labels[i] =
                                SendMessageW(get_child(hwnd, 130 + i as i32), 0xf0, 0, 0) == 1;
                            next.charts[i] =
                                SendMessageW(get_child(hwnd, 150 + i as i32), 0xf0, 0, 0) == 1;
                        }
                        next.ram_format = SendMessageW(get_child(hwnd, 135), 0x147, 0, 0).clamp(0, 2) as u32;
                        let mut order = [0u16; 128];
                        let count = GetWindowTextW(get_child(hwnd, 106), order.as_mut_ptr(), 128);
                        next.order = String::from_utf16_lossy(&order[..count as usize]);
                        next.startup = SendMessageW(get_child(hwnd, 107), 0xf0, 0, 0) == 1;
                        let index = SendMessageW(get_child(hwnd, 108), 0x147, 0, 0);
                        if let Some(display) = displays::list().get(index as usize) {
                            next.display = display.id.clone();
                        }
                        let position_index = SendMessageW(get_child(hwnd, 109), 0x147, 0, 0) as i32;
                        next.position = position_index - 1;
                        if next.position != 3 {
                            next.offset = -1;
                        }
                        next.locked = SendMessageW(get_child(hwnd, 112), 0xf0, 0, 0) == 1;
                        let mut face = [0u16; 128];
                        let count = GetWindowTextW(get_child(hwnd, 120), face.as_mut_ptr(), 128);
                        if count > 0 {
                            next.font = String::from_utf16_lossy(&face[..count as usize]);
                        }
                        next.transparent = SendMessageW(get_child(hwnd, 114), 0xf0, 0, 0) == 1;
                        next.palette =
                            SendMessageW(get_child(hwnd, 115), 0x147, 0, 0).clamp(0, 2) as u32;
                        let result = settings::Settings::parse(&next.serialize())
                            .and_then(|_| settings::save(&next))
                            .and_then(|_| set_startup(next.startup));
                        match result {
                            Ok(()) => {
                                let face = wide(&next.font);
                                let small = CreateFontW(
                                    -16,
                                    0,
                                    0,
                                    0,
                                    400,
                                    0,
                                    0,
                                    0,
                                    1,
                                    0,
                                    0,
                                    5,
                                    0,
                                    face.as_ptr(),
                                );
                                if !small.is_null() {
                                    if a.small != a.font {
                                        DeleteObject(a.small);
                                    }
                                    a.small = small;
                                }
                                a.config = next;
                                meter_transparency(a.meter, a.config.transparent, a.config.theme);
                                a.worker.configure(a.config.clone());
                                KillTimer(a.controller, 1);
                                SetTimer(a.controller, 1, a.config.interval * 1000, null_mut());
                                InvalidateRect(a.meter, std::ptr::null(), 0);
                                a.stale = true;
                                place(a);
                                DestroyWindow(hwnd);
                            }
                            Err(e) => {
                                MessageBoxW(
                                    hwnd,
                                    wide(&e).as_ptr(),
                                    wide("Settings were not saved").as_ptr(),
                                    0x10,
                                );
                            }
                        }
                    }
                    _ => {}
                }
                0
            }
            0xf => {
                let a = app(hwnd);
                let mut p: Paint = std::mem::zeroed();
                let dc = BeginPaint(hwnd, &mut p);
                let light = is_light(a.config.theme);
                let bg = if light { 0xf5f5f5 } else { 0x202020 };
                let fg = if light { 0x202020 } else { 0xf4f4f4 };
                let brush = CreateSolidBrush(bg);
                FillRect(dc, &p.rect, brush);
                DeleteObject(brush);
                SetTextColor(dc, fg);
                SetBkMode(dc, 1);
                SelectObject(dc, a.small);
                let s = &a.snapshot;
                if hwnd == a.meter {
                    // Fixed slots stop digits from shifting the neighbouring readings.
                    let names = ["CPU", "GPU", "RAM", "Download", "Upload"];
                    let selected = a
                        .config
                        .order
                        .split(',')
                        .filter_map(|name| names.iter().position(|v| *v == name))
                        .filter(|i| a.config.visible[*i])
                        .collect::<Vec<_>>();
                    let columns = if a.config.ram_format == 2 && a.config.visible[2] && a.config.width >= 480 { 3 } else { layout::columns(a.config.width) };
                    let cell_width = (a.config.width - 16) / columns as i32;
                    let widths = layout::row_widths(a.config.width, &selected);
                    let mut row_left = 8;
                    let muted = if light { 0x5f5f5f } else { 0xa0a0a0 };
                    let accent = match (a.config.palette, light) {
                        (1, false) => rgb(0x6c, 0xcb, 0x8a),
                        (1, true) => rgb(0x18, 0x80, 0x38),
                        (2, false) => rgb(0xf0, 0xb4, 0x4d),
                        (2, true) => rgb(0x9d, 0x5d, 0x00),
                        (_, false) => rgb(0x4c, 0xc2, 0xff),
                        (_, true) => rgb(0x0f, 0x6c, 0xbd),
                    };
                    for (slot, &i) in selected.iter().enumerate() {
                        let left = if columns == 5 { row_left } else { 8 + (slot % columns) as i32 * cell_width };
                        let width = if columns == 5 { widths[slot] } else { cell_width };
                        let right = left + width - 4;
                        row_left += width;
                        let cell_height = if columns == 2 { 15 } else { 20 };
                        let top = if columns == 5 {
                            (a.meter_height - cell_height) / 2
                        } else if columns == 2 {
                            1 + (slot / columns) as i32 * 15
                        } else {
                            3 + (slot / columns) as i32 * 21
                        };
                        let (prefix, value) = match i {
                            0 => ("CPU ", percentage(s.cpu)),
                            1 => ("GPU ", percentage(s.gpu)),
                            2 => ("RAM ", match (s.ram, a.config.ram_format) {
                                (Some(_), 1) => format!("{:.1} GiB", s.used as f64 / 1073741824.0),
                                (Some(v), 2) => format!("{v:.0}% {:.1} GiB", s.used as f64 / 1073741824.0),
                                _ => percentage(s.ram),
                            }),
                            3 => ("↓ ", rate(s.download, a.config.bits)),
                            _ => ("↑ ", rate(s.upload, a.config.bits)),
                        };
                        let prefix = if a.config.labels[i] { prefix } else { "" };
                        let full_text = wide(&format!("{prefix}{value}"));
                        let mut extent = [0; 2];
                        GetTextExtentPoint32W(dc, full_text.as_ptr(), (full_text.len() - 1) as i32, &mut extent);
                        let available = (right - left - if a.config.charts[i] { 18 } else { 0 }).max(1);
                        let fitted_font = if extent[0] > available {
                            let height = (16 * available / extent[0]).clamp(8, 16);
                            let face = wide(&a.config.font);
                            CreateFontW(-height, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, face.as_ptr())
                        } else {
                            null_mut()
                        };
                        if !fitted_font.is_null() {
                            SelectObject(dc, fitted_font);
                        }
                        let used = [s.cpu, s.gpu, s.ram].get(i).copied().flatten();
                        let value_color = match used {
                            Some(v) if v >= 95.0 => {
                                if light {
                                    rgb(0xc4, 0x2b, 0x1c)
                                } else {
                                    rgb(0xff, 0x6b, 0x6b)
                                }
                            }
                            Some(v) if v >= 85.0 => {
                                if light {
                                    rgb(0x9d, 0x5d, 0x00)
                                } else {
                                    rgb(0xff, 0xc8, 0x3d)
                                }
                            }
                            _ => fg,
                        };
                        let mut cell = Rect {
                            left,
                            top,
                            right,
                            bottom: top + cell_height,
                        };
                        SetTextColor(dc, if i >= 3 { accent } else { muted });
                        DrawTextW(dc, wide(prefix).as_ptr(), -1, &mut cell, 0x824);
                        let mut size = [0i32; 2];
                        GetTextExtentPoint32W(
                            dc,
                            wide(prefix).as_ptr(),
                            prefix.encode_utf16().count() as i32,
                            &mut size,
                        );
                        let mut cell = Rect {
                            left: left + size[0],
                            top,
                            right,
                            bottom: top + cell_height,
                        };
                        SetTextColor(dc, value_color);
                        DrawTextW(dc, wide(&value).as_ptr(), -1, &mut cell, 0x824);
                        if a.config.charts[i] {
                            let text = wide(&value);
                            let mut value_size = [0; 2];
                            GetTextExtentPoint32W(dc, text.as_ptr(), (text.len() - 1) as i32, &mut value_size);
                            let chart_left = left + size[0] + value_size[0] + 3;
                            if right - chart_left >= 8 {
                                draw_history(dc, &a.history, i, Rect { left: chart_left, top: top + 2, right, bottom: top + cell_height - 2 }, accent);
                            }
                        }
                        if !fitted_font.is_null() {
                            SelectObject(dc, a.small);
                            DeleteObject(fitted_font);
                        }
                    }
                } else if hwnd == a.details {
                    let name = a
                        .devices
                        .iter()
                        .find(|d| d.id == a.config.adapter)
                        .map(|d| d.name.as_str())
                        .unwrap_or("Select adapter");
                    let text = format!(
                        "SYSTEM OVERVIEW\n\nCPU busy time       {}\nGPU busiest engine  {}\nRAM                 {}  ({:.1} / {:.1} GiB)\nDownload            {}\nUpload              {}\n\n{} · {}\nGPU: {}\n{}",
                        percentage(s.cpu),
                        percentage(s.gpu),
                        percentage(s.ram),
                        s.used as f64 / 1073741824.0,
                        s.total as f64 / 1073741824.0,
                        rate(s.download, a.config.bits),
                        rate(s.upload, a.config.bits),
                        name,
                        s.network_status,
                        a.gpus
                            .iter()
                            .find(|(id, _)| id.eq_ignore_ascii_case(&s.gpu_id))
                            .map(|(_, name)| name.as_str())
                            .unwrap_or("Unavailable / selected GPU missing"),
                        if a.blocked {
                            "Meter hidden: insufficient taskbar space"
                        } else {
                            a.notice.as_str()
                        }
                    );
                    SetWindowTextW(get_child(hwnd, 600), wide(&text).as_ptr());
                    let names = ["CPU", "GPU", "RAM", "Download", "Upload"];
                    let mut row = 0;
                    for i in 0..5 {
                        if a.config.visible[i] && a.config.charts[i] {
                            let top = 265 + row * 40;
                            let mut label = Rect { left: 24, top, right: 115, bottom: top + 30 };
                            SetTextColor(dc, fg);
                            DrawTextW(dc, wide(names[i]).as_ptr(), -1, &mut label, 0x824);
                            draw_history(dc, &a.history, i, Rect { left: 120, top, right: 410, bottom: top + 30 }, if light { rgb(0x0f, 0x6c, 0xbd) } else { rgb(0x4c, 0xc2, 0xff) });
                            row += 1;
                        }
                    }
                }
                EndPaint(hwnd, &p);
                0
            }
            0x2b => {
                let item = &*(lp as *const DrawItem);
                if item.kind == 4 {
                    draw_button(item, is_light(app(hwnd).config.theme));
                    return 1;
                }
                DefWindowProcW(hwnd, msg, wp, lp)
            }
            0x100 if wp == 27 => {
                if hwnd != app(hwnd).meter {
                    DestroyWindow(hwnd);
                }
                0
            }
            0x6 => {
                let a = app(hwnd);
                if wp & 0xffff == 0 && hwnd == a.details && lp as H != a.settings {
                    ShowWindow(hwnd, 0);
                }
                0
            }
            0x10 => {
                DestroyWindow(hwnd);
                0
            }
            0x2 => {
                let a = app(hwnd);
                if hwnd == a.controller {
                    if !a.meter.is_null() {
                        DestroyWindow(a.meter);
                        a.meter = null_mut();
                    }
                    notify(hwnd, true);
                    PostQuitMessage(0);
                } else if hwnd == a.meter {
                    a.meter = null_mut();
                } else if hwnd == a.details {
                    a.details = null_mut();
                } else if hwnd == a.settings {
                    a.settings = null_mut();
                }
                0
            }
            0x138 | 0x135 | 0x133 | 0x134 => {
                let light = is_light(app(hwnd).config.theme);
                SetTextColor(wp as H, if light { 0x202020 } else { 0xf4f4f4 });
                #[link(name = "gdi32")]
                unsafe extern "system" {
                    fn SetDCBrushColor(dc: H, color: u32) -> u32;
                    fn GetStockObject(index: i32) -> H;
                    fn SetBkColor(dc: H, color: u32) -> u32;
                }
                let color = if light { 0xf5f5f5 } else { 0x202020 };
                SetBkColor(wp as H, color);
                SetDCBrushColor(wp as H, color);
                return GetStockObject(18) as isize;
            }
            0x218 => {
                let a = app(hwnd);
                if wp == 7 || wp == 18 {
                    a.worker.configure(a.config.clone());
                    a.snapshot = Default::default();
                }
                1
            }
            0x1a | 0x7e => {
                place(app(hwnd));
                0
            }
            _ => {
                if msg == RegisterWindowMessageW(w!("TaskbarCreated").as_ptr()) {
                    place(app(hwnd));
                    notify(hwnd, false);
                    let a = app(hwnd);
                    if !a.hook.is_null() {
                        UnhookWinEvent(a.hook);
                    }
                    a.hook = install_shell_hook();
                    return 0;
                }
                DefWindowProcW(hwnd, msg, wp, lp)
            }
        }
    }
}
fn main() {
    unsafe {
        if std::env::args().any(|a| a == "--sample") {
            let devices = sampler::adapters();
            for d in &devices {
                if d.physical {
                    println!("Adapter {}: {} active={}", d.id, d.name, d.active);
                }
            }
            let mut s = sampler::Sampler::new(
                devices
                    .iter()
                    .find(|d| d.active && d.physical)
                    .map(|d| d.id)
                    .unwrap_or(0),
                String::new(),
            );
            for _ in 0..3 {
                println!("{:?}", s.sample());
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            return;
        }
        SetProcessDPIAware();
        let mutex = CreateMutexW(null_mut(), 0, w!("Local\\TaskbarSystemMonitor").as_ptr());
        if GetLastError() == 183 {
            let old = FindWindowW(w!("TaskbarSystemMonitor").as_ptr(), std::ptr::null());
            if !old.is_null() {
                SendMessageW(old, 0x202, 0, 0);
            }
            return;
        }
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let instance = GetModuleHandleW(std::ptr::null());
        for name in ["TaskbarSystemMonitor", "MonitorPanel", "KurmonMeter"] {
            let class_name = wide(name);
            let class = Class {
                style: 0,
                proc: Some(proc),
                a: 0,
                b: 0,
                instance,
                icon: LoadIconW(instance, 101usize as *const u16),
                cursor: LoadCursorW(null_mut(), 32512usize as *const u16),
                background: null_mut(),
                menu: std::ptr::null(),
                name: class_name.as_ptr(),
            };
            RegisterClassW(&class);
        }
        let devices = sampler::adapters();
        let (mut config, notice) = match settings::load() {
            Ok(c) => (c, String::new()),
            Err(e) => (
                settings::Settings::default(),
                format!("Settings recovery: {e}; original file preserved"),
            ),
        };
        if config.width == 340 {
            config.width = 500;
        }
        if config.adapter == 0 {
            config.adapter = devices
                .iter()
                .find(|d| d.active && d.physical)
                .map(|d| d.id)
                .unwrap_or(0);
        }
        let gpus = sampler::gpus();
        if !gpus
            .iter()
            .any(|(_, name)| name.eq_ignore_ascii_case(&config.gpu))
        {
            // A saved name that is gone (driver change, eGPU removed) must not blank the reading.
            config.gpu = gpus
                .first()
                .map(|(_, name)| name.clone())
                .unwrap_or_default();
        }
        let face = wide(&config.font);
        let font = CreateFontW(-16, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, face.as_ptr());
        let mut a = Box::new(App {
            worker: worker::Worker::new(config.clone()),
            config,
            snapshot: Default::default(),
            history: history::History::new(),
            history_stamp: None,
            devices,
            gpus,
            font,
            small: font,
            details: null_mut(),
            settings: null_mut(),
            controller: null_mut(),
            meter: null_mut(),
            parent: null_mut(),
            rejected: null_mut(),
            retries: 0,
            scan: scan::Scanner::new(),
            geometry: None,
            scan_tries: 0,
            stale: true,
            visible: true,
            meter_height: 60,
            blocked: false,
            last: Instant::now(),
            notice,
            hook: null_mut(),
            foreground_hook: null_mut(),
        });
        let hwnd = CreateWindowExW(
            0x08000080,
            w!("TaskbarSystemMonitor").as_ptr(),
            w!("Kurmon").as_ptr(),
            0x80000000,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        );
        a.controller = hwnd;
        SetWindowLongPtrW(hwnd, -21, &mut *a as *mut App as isize);
        place(&mut a);
        notify(hwnd, false);
        SetTimer(hwnd, 1, a.config.interval * 1000, null_mut());
        a.hook = install_shell_hook();
        a.foreground_hook = SetWinEventHook(3, 3, null_mut(), Some(shell_changed), 0, 0, 0);
        let mut m: Message = std::mem::zeroed();
        while GetMessageW(&mut m, null_mut(), 0, 0) > 0 {
            if m.msg == 0x100 && m.wp == 27 {
                let panel = if !a.settings.is_null() {
                    a.settings
                } else {
                    a.details
                };
                if !panel.is_null() {
                    DestroyWindow(panel);
                    continue;
                }
            }
            if m.msg == 0x100
                && m.wp == 13
                && !a.settings.is_null()
                && GetFocus() == get_child(a.settings, 201)
            {
                SendMessageW(a.settings, 0x111, 201, 0);
                continue;
            }
            if (!a.settings.is_null() && IsDialogMessageW(a.settings, &mut m) != 0)
                || (!a.details.is_null() && IsDialogMessageW(a.details, &mut m) != 0)
            {
                if !a.settings.is_null() {
                    let focus = GetFocus();
                    if IsChild(a.settings, focus) != 0 {
                        let mut rect = Rect::default();
                        let mut client = Rect::default();
                        GetWindowRect(focus, &mut rect);
                        GetClientRect(a.settings, &mut client);
                        let mut top = [rect.left, rect.top];
                        let mut bottom = [rect.right, rect.bottom];
                        ScreenToClient(a.settings, &mut top);
                        ScreenToClient(a.settings, &mut bottom);
                        let pos = GetScrollPos(a.settings, 1);
                        if top[1] < 0 {
                            scroll_settings(a.settings, pos + top[1]);
                        } else if bottom[1] > client.bottom {
                            scroll_settings(a.settings, pos + bottom[1] - client.bottom);
                        }
                    }
                }
                continue;
            }
            TranslateMessage(&m);
            DispatchMessageW(&m);
        }
        if !a.hook.is_null() {
            UnhookWinEvent(a.hook);
        }
        if !a.foreground_hook.is_null() {
            UnhookWinEvent(a.foreground_hook);
        }
        if a.small != a.font {
            DeleteObject(a.small);
        }
        DeleteObject(a.font);
        CloseHandle(mutex);
        CoUninitialize();
    }
}
