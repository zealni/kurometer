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
mod temperature;
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
    fn FindWindowExW(parent: H, after: H, class: *const u16, title: *const u16) -> H;
    fn IsChild(parent: H, child: H) -> i32;
    fn IsWindow(hwnd: H) -> i32;
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
    fn SystemParametersInfoW(action: u32, param: u32, data: *mut c_void, flags: u32) -> i32;
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
    fn SetProcessDpiAwarenessContext(context: H) -> i32;
    fn SetThreadDpiAwarenessContext(context: H) -> H;
    fn MonitorFromWindow(hwnd: H, flags: u32) -> H;
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
    fn GetCursorPos(p: *mut [i32; 2]) -> i32;
    fn SetForegroundWindow(hwnd: H) -> i32;
    fn EnableWindow(hwnd: H, enable: i32) -> i32;
    fn SetFocus(hwnd: H) -> H;
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
#[link(name = "shcore")]
unsafe extern "system" {
    fn GetDpiForMonitor(monitor: H, kind: i32, x: *mut u32, y: *mut u32) -> i32;
}

struct DpiContext(H);
impl DpiContext {
    fn new(context: isize) -> Self {
        unsafe {
            let mut previous = SetThreadDpiAwarenessContext(context as H);
            if previous.is_null() && context == -4 {
                previous = SetThreadDpiAwarenessContext(-3isize as H);
            }
            Self(previous)
        }
    }
}
impl Drop for DpiContext {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { SetThreadDpiAwarenessContext(self.0); }
        }
    }
}

fn scale_pixels(value: i32, numerator: u32, denominator: u32) -> i32 {
    let value = i64::from(value) * i64::from(numerator);
    let denominator = i64::from(denominator.max(1));
    ((value + denominator / 2) / denominator).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn taskbar_dpi(bar: H) -> Option<u32> {
    unsafe {
        let monitor = MonitorFromWindow(bar, 0);
        let (mut x, mut y) = (0, 0);
        if monitor.is_null() || GetDpiForMonitor(monitor, 0, &mut x, &mut y) < 0 || x == 0 {
            None
        } else {
            Some(x)
        }
    }
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
    fn Polygon(dc: H, points: *const [i32; 2], count: i32) -> i32;
    fn Ellipse(dc: H, left: i32, top: i32, right: i32, bottom: i32) -> i32;
    fn GetStockObject(index: i32) -> H;
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
    fn SaveDC(dc: H) -> i32;
    fn RestoreDC(dc: H, saved: i32) -> i32;
    fn IntersectClipRect(dc: H, left: i32, top: i32, right: i32, bottom: i32) -> i32;
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
        fonts.retain(|name| [
            "Segoe UI Variable", "Segoe UI", "Arial", "Tahoma", "Verdana",
            "Consolas", "Cascadia Code", "Cascadia Mono",
        ].iter().any(|common| name.eq_ignore_ascii_case(common)));
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
fn draw_ram_history(dc: H, history: &history::History, rect: Rect, light: bool) {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width < 2 || height < 2 { return; }
    unsafe {
        let background = CreateSolidBrush(if light { rgb(0xf5, 0xf8, 0xfb) } else { rgb(0x17, 0x17, 0x17) });
        FillRect(dc, &rect, background);
        DeleteObject(background);

        let fill = CreateSolidBrush(if light { rgb(0xd3, 0xe8, 0xf7) } else { rgb(0x0c, 0x22, 0x38) });
        let old_brush = SelectObject(dc, fill);
        let old_pen = SelectObject(dc, GetStockObject(8));
        let count = history.points(2).count();
        let mut points = [[0i32; 2]; 62];
        let mut len = 0usize;
        for (i, value) in history.points(2).chain(std::iter::once(None)).enumerate() {
            if let Some(value) = value.filter(|value| value.is_finite()) {
                let x = rect.right - 1 - ((count - 1 - i) as i32 * (width - 1) / 59);
                let y = rect.bottom - 1 - (value.clamp(0.0, 100.0) / 100.0 * (height - 1) as f64) as i32;
                if len == 0 { points[0] = [x, rect.bottom - 1]; len = 1; }
                points[len] = [x, y];
                len += 1;
            } else if len > 1 {
                points[len] = [points[len - 1][0], rect.bottom - 1];
                Polygon(dc, points.as_ptr(), (len + 1) as i32);
                len = 0;
            }
        }
        SelectObject(dc, old_brush);
        SelectObject(dc, old_pen);
        DeleteObject(fill);

        let grid = CreatePen(0, 1, if light { rgb(0xc8, 0xd2, 0xda) } else { rgb(0x31, 0x39, 0x40) });
        let old = SelectObject(dc, grid);
        for i in 0..=10 {
            let y = rect.top + i * (height - 1) / 10;
            MoveToEx(dc, rect.left, y, null_mut());
            LineTo(dc, rect.right, y);
        }
        for i in 0..=20 {
            let x = rect.left + i * (width - 1) / 20;
            MoveToEx(dc, x, rect.top, null_mut());
            LineTo(dc, x, rect.bottom);
        }
        SelectObject(dc, old);
        DeleteObject(grid);
    }
    draw_history(dc, history, 2, rect, if light { rgb(0x0f, 0x6c, 0xbd) } else { rgb(0x4c, 0xc2, 0xff) });
}

fn draw_metric_icon(dc: H, metric: usize, rect: Rect, color: u32, dpi: u32) -> i32 {
    let size = scale_pixels(12, dpi, 96).min((rect.bottom - rect.top - 4).max(0));
    if size < 6 {
        return 0;
    }
    let left = rect.left + scale_pixels(2, dpi, 96);
    let top = rect.top + (rect.bottom - rect.top - size) / 2;
    let right = left + size;
    let bottom = top + size;
    unsafe {
        let pen = CreatePen(0, scale_pixels(1, dpi, 96), color);
        if pen.is_null() {
            return 0;
        }
        let old_pen = SelectObject(dc, pen);
        let old_brush = SelectObject(dc, GetStockObject(5));
        let path = |points: &[(i32, i32)]| {
            if let Some(&(x, y)) = points.first() {
                MoveToEx(dc, x, y, null_mut());
                for &(x, y) in &points[1..] {
                    LineTo(dc, x, y);
                }
            }
        };
        match metric {
            0 => {
                let inset = size / 4;
                path(&[
                    (left + inset, top + inset), (right - inset, top + inset),
                    (right - inset, bottom - inset), (left + inset, bottom - inset),
                    (left + inset, top + inset),
                ]);
                for offset in [size / 3, size * 2 / 3] {
                    path(&[(left, top + offset), (left + inset, top + offset)]);
                    path(&[(right - inset, top + offset), (right, top + offset)]);
                    path(&[(left + offset, top), (left + offset, top + inset)]);
                    path(&[(left + offset, bottom - inset), (left + offset, bottom)]);
                }
            }
            1 | 5..DISK_WRITE => {
                path(&[(left, top + 1), (right, top + 1), (right, bottom - 3), (left, bottom - 3), (left, top + 1)]);
                path(&[(left + size / 2, bottom - 3), (left + size / 2, bottom), (right - 2, bottom)]);
            }
            2 => {
                path(&[(left, top + 1), (right, top + 1), (right, bottom - 1), (left, bottom - 1), (left, top + 1)]);
                for x in [left + size / 3, left + size * 2 / 3] {
                    path(&[(x, top + size / 4), (x, bottom - size / 4)]);
                }
            }
            3 | DISK_READ => {
                path(&[(left + size / 2, top), (left + size / 2, bottom - 2)]);
                path(&[(left + 2, bottom - size / 3), (left + size / 2, bottom - 1), (right - 2, bottom - size / 3)]);
            }
            4 | DISK_WRITE => {
                path(&[(left + size / 2, bottom), (left + size / 2, top + 2)]);
                path(&[(left + 2, top + size / 3), (left + size / 2, top + 1), (right - 2, top + size / 3)]);
            }
            CPU_CLOCK => {
                Ellipse(dc, left, top, right, bottom);
                path(&[(left + size / 2, top + 2), (left + size / 2, top + size / 2), (right - 2, top + size / 2)]);
            }
            _ => {}
        }
        SelectObject(dc, old_brush);
        SelectObject(dc, old_pen);
        DeleteObject(pen);
    }
    size + scale_pixels(6, dpi, 96)
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn commit_startup<T>(
    previous: T,
    apply: impl FnOnce() -> Result<(), String>,
    save: impl FnOnce() -> Result<(), String>,
    rollback: impl FnOnce(T) -> Result<(), String>,
) -> Result<(), String> {
    match apply().and_then(|_| save()) {
        Ok(()) => Ok(()),
        Err(error) => match rollback(previous) {
            Ok(()) => Err(error),
            Err(rollback) => Err(format!(
                "{error}\n\nCould not restore the previous startup registry value: {rollback}\nStartup and saved settings may now differ; check the startup entry before retrying."
            )),
        },
    }
}

fn save_with_startup(next: &settings::Settings) -> Result<(), String> {
    use windows::Win32::System::Registry::*;
    settings::load()?;
    let executable = if next.startup {
        Some(std::env::current_exe().map_err(|e| e.to_string())?)
    } else {
        None
    };
    struct Key(HKEY);
    impl Drop for Key {
        fn drop(&mut self) {
            unsafe { let _ = RegCloseKey(self.0); }
        }
    }
    unsafe {
        let mut handle = HKEY::default();
        let status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            None,
            KEY_QUERY_VALUE | KEY_SET_VALUE,
            &mut handle,
        );
        if status.0 == 2 {
            if !next.startup {
                return settings::save(next);
            }
            // shortcut: a failed save may leave an empty Run key; use coordinated transactions if key absence must be restored.
            RegCreateKeyW(
                HKEY_CURRENT_USER,
                w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
                &mut handle,
            ).ok().map_err(|e| e.to_string())?;
        } else {
            status.ok().map_err(|e| e.to_string())?;
        }
        let key = Key(handle);
        let mut kind = REG_VALUE_TYPE::default();
        let mut size = 0u32;
        let status = RegQueryValueExW(key.0, w!("TaskbarSystemMonitor"), None, Some(&mut kind), None, Some(&mut size));
        let previous = if status.0 == 2 {
            None
        } else {
            status.ok().map_err(|e| e.to_string())?;
            let mut bytes = vec![0; size as usize];
            RegQueryValueExW(key.0, w!("TaskbarSystemMonitor"), None, Some(&mut kind), Some(bytes.as_mut_ptr()), Some(&mut size))
                .ok().map_err(|e| e.to_string())?;
            bytes.truncate(size as usize);
            Some((kind, bytes))
        };
        let write_value = |value: Option<(REG_VALUE_TYPE, Vec<u8>)>| -> Result<(), String> {
            let deleting = value.is_none();
            let status = match value {
                Some((kind, bytes)) => RegSetValueExW(key.0, w!("TaskbarSystemMonitor"), Some(0), kind, Some(&bytes)),
                None => RegDeleteValueW(key.0, w!("TaskbarSystemMonitor")),
            };
            if status.0 == 2 && deleting {
                Ok(())
            } else {
                status.ok().map_err(|e| e.to_string())
            }
        };
        let value = executable.map(|path| {
            let text = wide(&format!("\"{}\"", path.display()));
            let bytes = std::slice::from_raw_parts(text.as_ptr().cast::<u8>(), text.len() * 2).to_vec();
            (REG_SZ, bytes)
        });
        if previous == value {
            settings::save(next)
        } else {
            commit_startup(previous, || write_value(value), || settings::save(next), |value| write_value(value))
        }
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
        let controller = FindWindowW(w!("TaskbarSystemMonitor").as_ptr(), std::ptr::null());
        if controller.is_null() {
            return;
        }
        let state = GetWindowLongPtrW(controller, -21) as *const App;
        if !state.is_null() {
            let meter = (*state).meter;
            if (!meter.is_null() && (target == meter || IsChild(meter, target) != 0))
                || (*state).extra_meters.iter().any(|meter| target == meter.hwnd || IsChild(meter.hwnd, target) != 0)
            {
                return;
            }
        }
        let taskbar_event = displays::list()
            .iter()
            .any(|d| target == d.taskbar || IsChild(d.taskbar, target) != 0);
        if layout::relevant_shell_event(_event, taskbar_event) {
            PostMessageW(controller, if _event == 3 { 0x8003 } else { 0x8002 }, 0, 0);
        }
    }
}
struct Meter {
    id: String,
    hwnd: H,
    parent: H,
    geometry: Option<scan::Layout>,
    generation: u64,
    context: Option<(usize, (i32, i32, i32, i32), u32)>,
    pending: Option<scan::Request>,
    dpi: u32,
    blocked: bool,
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
    heading: H,
    details: H,
    details_owner: H,
    settings: H,
    settings_original: Option<settings::Settings>,
    settings_dirty: bool,
    choice: H,
    choice_result: i32,
    choice_menu: bool,
    controller: H,
    meter: H,
    parent: H,
    rejected: H,
    retries: u8,
    scan: scan::Scanner,
    geometry: Option<scan::Layout>,
    scan_generation: u64,
    scan_context: Option<(usize, (i32, i32, i32, i32), u32)>,
    scan_pending: Option<scan::Request>,
    scan_timer: bool,
    meter_dpi: u32,
    drag_start: Option<Rect>,
    visible: bool,
    blocked: bool,
    extra_meters: Vec<Meter>,
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
        // Layered painting is redirected rather than sharing Explorer's child surface.
        SetWindowLongPtrW(hwnd, -20, (ex | 0x80000) as isize);
        let key = if is_light(theme) { 0x00f5f5f5 } else { 0x00202020 };
        SetLayeredWindowAttributes(hwnd, key, 255, if enabled { 1 } else { 2 });
        SetWindowPos(hwnd, null_mut(), 0, 0, 0, 0, 0x27);
    }
}
fn percentage(v: Option<f64>) -> String {
    v.map(|v| format!("{v:.0}%")).unwrap_or_else(|| "--".into())
}
fn cpu_clock(mhz: Option<f64>) -> String {
    mhz.map(|v| format!("{:.2} GHz", v / 1000.0)).unwrap_or_else(|| "--".into())
}

fn ram_value(ram: Option<f64>, used: u64, format: u32) -> String {
    match (ram, format) {
        (Some(_), 1) => format!("{:.1} GiB", used as f64 / 1073741824.0),
        (Some(v), 2) => format!("{v:.0}% · {:.1} GiB", used as f64 / 1073741824.0),
        _ => percentage(ram),
    }
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
fn accept_snapshot(
    config: &settings::Settings,
    current: &mut sampler::Snapshot,
    next: sampler::Snapshot,
) -> bool {
    let changed = percentage(next.cpu) != percentage(current.cpu)
        || percentage(next.gpu) != percentage(current.gpu)
        || percentage(next.ram) != percentage(current.ram)
        || (config.ram_format != 0
            && format!("{:.1}", next.used as f64 / 1073741824.0)
                != format!("{:.1}", current.used as f64 / 1073741824.0))
        || rate(next.download, config.bits) != rate(current.download, config.bits)
        || rate(next.upload, config.bits) != rate(current.upload, config.bits)
        || (config.all_gpus && next.gpus != current.gpus)
        || (config.gpu_temperature && next.gpu_temperatures != current.gpu_temperatures)
        || (config.cpu_speed && cpu_clock(next.cpu_mhz) != cpu_clock(current.cpu_mhz))
        || (config.disk_activity
            && (rate(next.disk_read, false) != rate(current.disk_read, false)
                || rate(next.disk_write, false) != rate(current.disk_write, false)));
    *current = next;
    changed
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
fn settings_control_position(x: i32, y: i32, wide: bool) -> (i32, i32) {
    let (x, y) = if wide && (466..=900).contains(&y) {
        (x + 480, y - 450)
    } else if wide && y == 910 {
        (x, 500)
    } else {
        (x, y)
    };
    (x, y + 32)
}

fn settings_content_size(hwnd: H) -> (i32, i32) {
    let mut rect = Rect::default();
    unsafe { GetWindowRect(hwnd, &mut rect); }
    if rect.right - rect.left >= 1000 { (960, 582) } else { (500, 992) }
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
        let a = app(parent);
        let (x, y) = if parent == a.settings {
            settings_control_position(x, y, settings_content_size(parent).0 == 960)
        } else {
            (x, y)
        };
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
unsafe fn create_meter(a: &mut App, bar: H, primary: bool) -> H {
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
        if GetParent(meter) != bar {
            let style = GetWindowLongPtrW(meter, -16) as u32;
            SetWindowLongPtrW(meter, -16, ((style & !0x40000000) | 0x80000000) as isize);
        }
        if primary {
            if GetParent(meter) == bar {
                a.parent = bar;
                a.retries = 0;
                KillTimer(a.controller, 4);
            } else {
                // The top-level companion remains usable until parenting can be retried.
                a.parent = null_mut();
                a.rejected = bar;
                a.retries = a.retries.saturating_add(1);
                if a.retries <= 6 { SetTimer(a.controller, 4, 5000, null_mut()); }
            }
        }
        meter
    }
}
fn keep_meter_during_scan(a: &App) -> bool {
    let taskbar = if !a.parent.is_null() { a.parent } else { a.scan_context.map(|(bar, _, _)| bar as H).unwrap_or(null_mut()) };
    let mut current = Rect::default();
    let taskbar_alive = !taskbar.is_null() && unsafe { IsWindow(taskbar) != 0 && GetWindowRect(taskbar, &mut current) != 0 };
    let previous = a.scan_context.map(|(_, bounds, _)| bounds);
    let bounds_match = layout::taskbar_bounds_match(previous, (current.left, current.top, current.right, current.bottom));
    layout::keep_meter_visible_during_refresh(!a.parent.is_null(), taskbar_alive, bounds_match)
}
fn invalidate_geometry(a: &mut App) {
    a.scan_generation = a.scan_generation.wrapping_add(1);
    a.geometry = None;
    a.scan_pending = None;
    if !a.meter.is_null() && !keep_meter_during_scan(a) {
        unsafe { ShowWindow(a.meter, 0); }
    }
}

fn ensure_scan_timer(a: &mut App) {
    if !a.scan_timer {
        a.scan_timer = unsafe { SetTimer(a.controller, 3, 200, null_mut()) != 0 };
    }
}
fn apply_scan_results(a: &mut App) {
    let results: Vec<_> = std::iter::from_fn(|| a.scan.latest()).collect();
    for result in results {
        let request = result.request;
        if a.scan_pending == Some(request) {
            a.scan_pending = None;
            a.geometry = result.layout(a.scan_generation, request.bar, request.bounds);
        } else if let Some(meter) = a.extra_meters.iter_mut().find(|meter| meter.pending == Some(request)) {
            meter.pending = None;
            meter.geometry = result.layout(meter.generation, request.bar, request.bounds);
        }
    }
}

const CPU_CLOCK: usize = usize::MAX;
const DISK_READ: usize = usize::MAX - 1;
const DISK_WRITE: usize = usize::MAX - 2;

fn selected_metrics(config: &settings::Settings, gpus: &[(String, String)]) -> Vec<usize> {
    let names = ["CPU", "GPU", "RAM", "Download", "Upload"];
    let mut result = Vec::new();
    for name in config.order.split(',') {
        if let Some(i) = names.iter().position(|v| *v == name).filter(|i| config.visible[*i]) {
            result.push(i);
            if i == 1 && config.all_gpus {
                let selected = gpus.iter().position(|(_, name)| name.eq_ignore_ascii_case(&config.gpu));
                result.extend((0..gpus.len()).filter(|i| Some(*i) != selected).map(|i| i + 5));
            }
        }
    }
    if config.cpu_speed { result.push(CPU_CLOCK); }
    if config.disk_activity { result.extend([DISK_READ, DISK_WRITE]); }
    result
}

unsafe extern "system" fn hit_proc(hwnd: H, msg: u32, wp: usize, lp: isize) -> isize {
    unsafe {
        match msg {
            0x84 => {
                let parent = GetParent(hwnd);
                let a = app(parent);
                if parent == a.meter && a.parent.is_null() && !a.config.locked {
                    let mut r = Rect::default();
                    GetWindowRect(parent, &mut r);
                    let x = (lp as u32 as u16 as i16) as i32;
                    if x < r.left + 8 || x >= r.right - 8 { return -1; }
                }
                1
            }
            0x21 => 3,
            0x202 => { details(GetParent(hwnd)); 0 }
            0x205 => { show_menu(GetParent(hwnd)); 0 }
            0xf => {
                let mut paint: Paint = std::mem::zeroed();
                let dc = BeginPaint(hwnd, &mut paint);
                let brush = CreateSolidBrush(0);
                FillRect(dc, &paint.rect, brush);
                DeleteObject(brush);
                EndPaint(hwnd, &paint);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wp, lp),
        }
    }
}

unsafe fn update_hit_surface(meter: H, enabled: bool) {
    unsafe {
        let mut hit = FindWindowExW(meter, null_mut(), w!("KurmonHit").as_ptr(), std::ptr::null());
        if enabled && hit.is_null() {
            hit = CreateWindowExW(0x08080000, w!("KurmonHit").as_ptr(), std::ptr::null(), 0x40000000,
                0, 0, 1, 1, meter, null_mut(), GetModuleHandleW(std::ptr::null()), null_mut());
            if !hit.is_null() { SetLayeredWindowAttributes(hit, 0, 1, 2); }
        }
        if !hit.is_null() {
            let mut client = Rect::default();
            GetClientRect(meter, &mut client);
            SetWindowPos(hit, null_mut(), 0, 0, client.right, client.bottom, 0x14);
            ShowWindow(hit, if enabled { 4 } else { 0 });
        }
    }
}

unsafe fn place(a: &mut App) {
    let _dpi_context = DpiContext::new(-4);
    unsafe {
        let displays: Vec<_> = displays::list().into_iter().filter(|display| !display.taskbar.is_null()).collect();
        let targets = layout::display_targets(&a.config.display, &displays.iter().map(|d| (d.id.as_str(), d.primary)).collect::<Vec<_>>());
        let primary_index = displays.iter().position(|d| d.primary);
        if primary_index.map(|index| !targets.contains(&index)).unwrap_or(true) {
            if !a.meter.is_null() { ShowWindow(a.meter, 0); }
            if !a.meter.is_null() { DestroyWindow(a.meter); a.meter = null_mut(); a.parent = null_mut(); }
            a.blocked = false;
            reconcile_extra_meters(a);
            return;
        }
        if let Some(display) = primary_index.and_then(|index| displays.get(index)) {
            if !fullscreen(display.screen) {
                let bar = display.taskbar;
                let mut r = Rect::default();
                if bar.is_null() || GetWindowRect(bar, &mut r) == 0 {
                    if a.scan_context.take().is_some() {
                        invalidate_geometry(a);
                    }
                    ensure_scan_timer(a);
                    return;
                }
                let Some(dpi) = taskbar_dpi(bar) else {
                    invalidate_geometry(a);
                    a.blocked = true;
                    ensure_scan_timer(a);
                    return;
                };
                let bounds = (r.left, r.top, r.right, r.bottom);
                if a.scan_context != Some((bar as usize, bounds, dpi)) {
                    invalidate_geometry(a);
                    a.scan_context = Some((bar as usize, bounds, dpi));
                }
                let attached = GetParent(a.meter);
                if layout::recreate_meter(attached as usize, bar as usize, !a.meter.is_null(), a.rejected as usize) {
                    if !a.meter.is_null() {
                        DestroyWindow(a.meter);
                        a.meter = null_mut();
                    }
                    let created = create_meter(a, bar, true);
                    a.meter = created;
                    if created.is_null() {
                        ensure_scan_timer(a);
                        return;
                    }
                    SetWindowPos(created, null_mut(), 0, 0, 0, 0, 0x37);
                }
                let meter = a.meter;
                apply_scan_results(a);
                if a.geometry.is_none() {
                    let request = scan::Request { bar: bar as usize, generation: a.scan_generation, bounds };
                    if a.scan_pending.is_none() && a.scan.request(request) {
                        a.scan_pending = Some(request);
                    }
                    ensure_scan_timer(a);
                    if !keep_meter_during_scan(a) { ShowWindow(meter, 0); }
                    return;
                }
                let geometry = a.geometry.as_ref().unwrap();
                KillTimer(a.controller, 3);
                a.scan_timer = false;
                let mut width = scale_pixels(a.config.width, dpi, 96);
                let offset = if a.config.offset < 0 { -1 } else { scale_pixels(a.config.offset, dpi, 96) };
                if a.config.position == -1 {
                    let mut low = 0;
                    let mut high = width;
                    while low < high {
                        let candidate = low + (high - low + 1) / 2;
                        if layout::meter_position(bounds, candidate, -1, offset, &geometry.occupied).is_some() {
                            low = candidate;
                        } else {
                            high = candidate - 1;
                        }
                    }
                    width = low;
                }
                let Some((x, y, height)) = layout::meter_position(
                    (r.left, r.top, r.right, r.bottom),
                    width,
                    a.config.position,
                    offset,
                    &geometry.occupied,
                ) else {
                    a.blocked = true;
                    ShowWindow(meter, 0);
                    return;
                };
                let selected = selected_metrics(&a.config, &a.gpus);
                let meter_layout = layout::responsive_meter(width, height, dpi, &selected, a.config.ram_format == 2, a.config.grid_columns);
                if meter_layout.font_px < scale_pixels(8, dpi, 96) {
                    a.blocked = true;
                    ShowWindow(meter, 0);
                    return;
                }
                a.blocked = false;
                a.meter_dpi = dpi;
                let (x, y, after) = if GetParent(meter) == bar {
                    let mut point = [x, y];
                    ScreenToClient(bar, &mut point);
                    (point[0], point[1], null_mut())
                } else {
                    (x, y, -1isize as H)
                };
                if SetWindowPos(meter, after, x, y, width, height, 0x10) == 0 {
                    a.blocked = true;
                    ShowWindow(meter, 0);
                    return;
                }
                let mut actual = Rect::default();
                let mut client = Rect::default();
                if GetWindowRect(meter, &mut actual) == 0 || GetClientRect(meter, &mut client) == 0
                    || actual.left < r.left || actual.top < r.top || actual.right > r.right || actual.bottom > r.bottom
                    || client.right <= 0 || client.bottom <= 0
                    || geometry.occupied.iter().any(|&(left, right)| actual.left < right && actual.right > left)
                    || layout::responsive_meter(client.right, client.bottom, dpi, &selected, a.config.ram_format == 2, a.config.grid_columns).font_px < scale_pixels(8, dpi, 96)
                {
                    a.blocked = true;
                    ShowWindow(meter, 0);
                    return;
                }
                update_hit_surface(meter, a.config.transparent);
                InvalidateRect(meter, std::ptr::null(), 0);
                ShowWindow(meter, if a.visible { 4 } else { 0 });
                return;
            }
        }
        if a.scan_context.take().is_some() {
            invalidate_geometry(a);
        }
        KillTimer(a.controller, 3);
        a.scan_timer = false;
    }
    unsafe { reconcile_extra_meters(a); }
}
fn invalidate_extra_geometry(a: &mut App) {
    for meter in &mut a.extra_meters {
        meter.generation = meter.generation.wrapping_add(1);
        meter.geometry = None;
        meter.pending = None;
        if unsafe { IsWindow(meter.parent) } == 0 {
            unsafe { ShowWindow(meter.hwnd, 0); }
        }
    }
}
unsafe fn reconcile_extra_meters(a: &mut App) {
    let displays: Vec<_> = displays::list().into_iter().filter(|display| !display.taskbar.is_null()).collect();
    let identities: Vec<_> = displays.iter().map(|d| (d.id.as_str(), d.primary)).collect();
    let targets = layout::display_targets(&a.config.display, &identities);
    let desired: Vec<_> = targets.into_iter().filter_map(|i| displays.get(i))
        .filter(|d| !d.taskbar.is_null() && !d.primary)
        .cloned().collect();
    let desired_ids: Vec<_> = desired.iter().map(|d| d.id.as_str()).collect();
    for i in (0..a.extra_meters.len()).rev() {
        if !desired_ids.contains(&a.extra_meters[i].id.as_str()) {
            let meter = a.extra_meters.remove(i);
            unsafe { DestroyWindow(meter.hwnd); }
        }
    }
    apply_scan_results(a);
    for display in desired {
        if fullscreen(display.screen) {
            unsafe { ShowWindow(a.extra_meters.iter().find(|meter| meter.id == display.id).map(|meter| meter.hwnd).unwrap_or(null_mut()), 0); }
            continue;
        }
        let Some(dpi) = taskbar_dpi(display.taskbar) else { continue };
        let mut bar_rect = Rect::default();
        if unsafe { GetWindowRect(display.taskbar, &mut bar_rect) } == 0 { continue; }
        let bounds = (bar_rect.left, bar_rect.top, bar_rect.right, bar_rect.bottom);
        if let Some(index) = a.extra_meters.iter().position(|m| m.id == display.id && (m.parent != display.taskbar || unsafe { IsWindow(m.hwnd) } == 0)) {
            let stale = a.extra_meters.remove(index);
            unsafe { DestroyWindow(stale.hwnd); }
        }
        let index = if let Some(i) = a.extra_meters.iter().position(|m| m.id == display.id) { i } else {
            let hwnd = unsafe { create_meter(a, display.taskbar, false) };
            if hwnd.is_null() { continue; }
            a.extra_meters.push(Meter { id: display.id.clone(), hwnd, parent: display.taskbar, geometry: None, generation: 0, context: None, pending: None, dpi, blocked: false });
            unsafe { SetWindowPos(hwnd, null_mut(), 0, 0, 0, 0, 0x37); }
            a.extra_meters.len() - 1
        };
        let meter = &mut a.extra_meters[index];
        if meter.context != Some((display.taskbar as usize, bounds, dpi)) {
            meter.generation = meter.generation.wrapping_add(1);
            meter.geometry = None;
            meter.pending = None;
            meter.context = Some((display.taskbar as usize, bounds, dpi));
        }
        let Some(geometry) = meter.geometry.as_ref() else {
            let request = scan::Request { bar: display.taskbar as usize, generation: meter.generation, bounds };
            if meter.pending.is_none() && a.scan.request(request) { meter.pending = Some(request); }
            ensure_scan_timer(a);
            continue;
        };
        let mut width = scale_pixels(a.config.width, dpi, 96);
        let offset = if a.config.offset < 0 { -1 } else { scale_pixels(a.config.offset, dpi, 96) };
        let position = if a.config.position != -1
            && layout::meter_position(bounds, width, a.config.position, offset, &geometry.occupied).is_none() {
            -1
        } else {
            a.config.position
        };
        if position == -1 {
            let mut low = 0;
            let mut high = width;
            while low < high {
                let candidate = low + (high - low + 1) / 2;
                if layout::meter_position(bounds, candidate, -1, offset, &geometry.occupied).is_some() { low = candidate; } else { high = candidate - 1; }
            }
            width = low;
        }
        let Some((x, y, height)) = layout::meter_position(bounds, width, position, offset, &geometry.occupied) else {
            meter.blocked = true;
            unsafe { ShowWindow(meter.hwnd, 0); }
            continue;
        };
        let selected = selected_metrics(&a.config, &a.gpus);
        if layout::responsive_meter(width, height, dpi, &selected, a.config.ram_format == 2, a.config.grid_columns).font_px < scale_pixels(8, dpi, 96) {
            meter.blocked = true;
            unsafe { ShowWindow(meter.hwnd, 0); }
            continue;
        }
        let (x, y) = if unsafe { GetParent(meter.hwnd) == display.taskbar } {
            let mut point = [x, y];
            unsafe { ScreenToClient(display.taskbar, &mut point); }
            (point[0], point[1])
        } else { (x, y) };
        unsafe {
            let after = if GetParent(meter.hwnd) == display.taskbar { null_mut() } else { -1isize as H };
            if SetWindowPos(meter.hwnd, after, x, y, width, height, 0x10) == 0 {
                meter.blocked = true;
                ShowWindow(meter.hwnd, 0);
                continue;
            }
        }
        let mut actual = Rect::default();
        let mut client = Rect::default();
        if unsafe { GetWindowRect(meter.hwnd, &mut actual) == 0 || GetClientRect(meter.hwnd, &mut client) == 0 }
            || actual.left < bounds.0 || actual.top < bounds.1 || actual.right > bounds.2 || actual.bottom > bounds.3
            || geometry.occupied.iter().any(|&(left, right)| actual.left < right && actual.right > left)
            || client.right <= 0 || client.bottom <= 0 {
            meter.blocked = true;
            unsafe { ShowWindow(meter.hwnd, 0); }
            continue;
        }
        unsafe { update_hit_surface(meter.hwnd, a.config.transparent); }
        unsafe {
            InvalidateRect(meter.hwnd, std::ptr::null(), 0);
            ShowWindow(meter.hwnd, if a.visible { 4 } else { 0 });
        }
        meter.dpi = dpi;
        meter.blocked = false;
    }
}
fn panel_anchor(a: &App, owner: H) -> H {
    for candidate in [owner, a.details_owner, a.meter] {
        if !candidate.is_null() && (candidate == a.meter || a.extra_meters.iter().any(|m| m.hwnd == candidate)) {
            return candidate;
        }
    }
    a.extra_meters.first().map(|m| m.hwnd).unwrap_or(null_mut())
}

fn gpu_status_lines(gpus: &[(String, String)], snapshot: &sampler::Snapshot) -> Vec<String> {
    gpus.iter().enumerate().map(|(index, (id, name))| {
        let value = snapshot.gpus.iter().find(|(gpu, _)| gpu.eq_ignore_ascii_case(id))
            .and_then(|(_, value)| *value);
        format!("GPU {} · {} · {}", index + 1, percentage(value), name)
    }).collect()
}

fn details_extra_height(config: &settings::Settings, gpu_count: usize) -> i32 {
    24 * (i32::from(config.cpu_speed) + i32::from(config.disk_activity))
        + 40 * gpu_count.max(1) as i32 - 24
}

fn details_content_height(config: &settings::Settings, gpu_count: usize) -> i32 {
    313 + details_extra_height(config, gpu_count)
        + config.visible.iter().zip(config.charts).enumerate()
            .filter(|(_, (visible, chart))| **visible && *chart)
            .map(|(i, _)| if i == 2 { 140 } else { 40 }).sum::<i32>()
}

fn position_details(a: &App, panel: H, owner: H) {
    let _dpi_context = DpiContext::new(-1);
    unsafe {
        let owner = panel_anchor(a, owner);
        let owner_id = if owner == a.meter { String::new() } else { a.extra_meters.iter().find(|m| m.hwnd == owner).map(|m| m.id.clone()).unwrap_or_default() };
        if let Some(display) = displays::selected(&owner_id) {
            let mut meter = Rect::default();
            let mut rect = Rect::default();
            if GetWindowRect(owner, &mut meter) != 0 && GetWindowRect(panel, &mut rect) != 0 {
                let work = Rect { left: display.work.left, top: display.work.top, right: display.work.right, bottom: display.work.bottom };
                let width = (rect.right - rect.left).min((work.right - work.left).max(1));
                let height = (details_content_height(&a.config, a.gpus.len()) + 48).min((work.bottom - work.top).max(1));
                let (x, y) = layout::popup_position(
                    (meter.left, meter.top, meter.right, meter.bottom),
                    (width, height),
                    (work.left, work.top, work.right, work.bottom),
                );
                SetWindowPos(panel, null_mut(), x, y, width, height, 0x4 | 0x10);
                let y = details_content_height(&a.config, a.gpus.len()) - 40 - GetScrollPos(panel, 1);
                let dx = GetScrollPos(panel, 0);
                SetWindowPos(get_child(panel, 501), null_mut(), 24 - dx, y, 0, 0, 0x15);
                SetWindowPos(get_child(panel, 502), null_mut(), 146 - dx, y, 0, 0, 0x15);
                SetWindowPos(get_child(panel, 620), null_mut(), 0, 0, 390, 68 + details_extra_height(&a.config, a.gpus.len()), 0x16);
                let mut client = Rect::default();
                for _ in 0..2 {
                    GetClientRect(panel, &mut client);
                    SetScrollRange(panel, 0, 0, (430 - client.right).max(0), 1);
                    SetScrollRange(panel, 1, 0, (details_content_height(&a.config, a.gpus.len()) - client.bottom).max(0), 1);
                }
            }
        }
    }
}
unsafe fn choice_dialog(a: &mut App, owner: H, title: &str, prompt: &str, labels: &[&str], menu: bool) -> i32 {
    unsafe {
        let (width, height) = if menu { (260, 265) } else { (410, 155) };
        let mut anchor = Rect::default();
        GetWindowRect(owner, &mut anchor);
        let mut point = [anchor.left + 20, anchor.top + 20];
        if menu { GetCursorPos(&mut point); }
        let displays = displays::list();
        if let Some(display) = displays::at_point(&displays, point) {
            point[0] = point[0].clamp(display.work.left, (display.work.right - width).max(display.work.left));
            point[1] = point[1].clamp(display.work.top, (display.work.bottom - height).max(display.work.top));
        }
        let dialog = CreateWindowExW(0x80, wide("MonitorPanel").as_ptr(), wide(title).as_ptr(),
            0x80c80000, point[0], point[1], width, height, a.controller, null_mut(),
            GetModuleHandleW(std::ptr::null()), null_mut());
        if dialog.is_null() { return 0; }
        SetWindowLongPtrW(dialog, -21, a as *mut App as isize);
        a.choice = dialog;
        a.choice_result = 0;
        a.choice_menu = menu;
        fluent(dialog, is_light(a.config.theme));
        if !prompt.is_empty() { control(dialog, "STATIC", prompt, 0, 20, 14, 360, 28, 0); }
        for (i, label) in labels.iter().enumerate() {
            let (x, y, w) = if menu { (12, 8 + i as i32 * 39, 218) }
                else { (15 + i as i32 * 125, 55, 115) };
            control(dialog, "BUTTON", label, 301 + i, x, y, w, 32, 0x10000 | 0xb);
        }
        if !menu { EnableWindow(owner, 0); }
        ShowWindow(dialog, 5);
        SetForegroundWindow(dialog);
        SetFocus(get_child(dialog, 301));
        let mut message: Message = std::mem::zeroed();
        while !a.choice.is_null() {
            let status = GetMessageW(&mut message, null_mut(), 0, 0);
            if status <= 0 {
                if status == 0 { PostQuitMessage(message.wp as i32); }
                break;
            }
            if message.msg == 0x100 && message.wp == 27 {
                DestroyWindow(dialog);
            } else if IsDialogMessageW(dialog, &mut message) == 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        if !a.choice.is_null() { DestroyWindow(dialog); }
        if !menu { EnableWindow(owner, 1); SetForegroundWindow(owner); }
        a.choice_result
    }
}

fn show_menu(hwnd: H) {
    unsafe {
        let locked = app(hwnd).config.locked;
        let cmd = choice_dialog(app(hwnd), hwnd, "Kurmon", "", &[
            "Open details", "Settings", "Show / hide meter", "Exit",
            if locked { "Unlock position" } else { "Lock position" },
        ], true);
        match cmd {
            1 => details(hwnd),
            2 => open_settings(hwnd),
            3 => {
                app(hwnd).visible = !app(hwnd).visible;
                place(app(hwnd));
                reconcile_extra_meters(app(hwnd));
            }
            4 => {
                let a = app(hwnd);
                if !a.settings.is_null() {
                    close_settings(a.settings);
                    if !a.settings.is_null() { return; }
                }
                DestroyWindow(a.controller);
            }
            5 => {
                let a = app(hwnd);
                if !a.settings.is_null() {
                    SendMessageW(get_child(a.settings, 112), 0x00f5, 0, 0);
                } else {
                    let mut next = a.config.clone();
                    next.locked = !next.locked;
                    match settings::load().and_then(|_| settings::save(&next)) {
                        Ok(()) => apply_settings(a, next),
                        Err(error) => {
                            MessageBoxW(hwnd, wide(&error).as_ptr(), wide("Position was not saved").as_ptr(), 0x10);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
fn details(hwnd: H) {
    let _dpi_context = DpiContext::new(-1);
    unsafe {
        let a = app(hwnd);
        a.details_owner = hwnd;
        if !a.details.is_null() {
            position_details(a, a.details, a.details_owner);
            ShowWindow(a.details, 5);
            SetForegroundWindow(a.details);
            return;
        }
        let h = CreateWindowExW(
            0x80,
            wide("MonitorPanel").as_ptr(),
            wide("Kurmon overview").as_ptr(),
            0x80f80000,
            100,
            500,
            470,
            details_content_height(&a.config, a.gpus.len()) + 48,
            a.controller,
            null_mut(),
            GetModuleHandleW(std::ptr::null()),
            null_mut(),
        );
        SetWindowLongPtrW(h, -21, a as *mut App as isize);
        a.details = h;
        fluent(h, is_light(a.config.theme));
        let heading = control(h, "STATIC", "System overview", 600, 24, 16, 390, 24, 0);
        SendMessageW(heading, 0x30, a.heading as usize, 1);
        for (i, label) in ["CPU busy time", "GPU busiest engine", "RAM", "Download", "Upload"].iter().enumerate() {
            let y = 48 + i as i32 * 27;
            control(h, "STATIC", label, 0, 24, y, 190, 22, 0);
            let value = control(h, "STATIC", "--", 610 + i, 220, y - 2, 190, 26, 0);
            SendMessageW(value, 0x30, a.heading as usize, 1);
        }
        control(h, "STATIC", "", 620, 24, 192, 390, 68, 0);
        control(
            h,
            "BUTTON",
            "Settings",
            501,
            24,
            details_content_height(&a.config, a.gpus.len()) - 40,
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
            details_content_height(&a.config, a.gpus.len()) - 40,
            130,
            32,
            0x10000 | 0xb,
        );
        position_details(a, h, a.details_owner);
        ShowWindow(h, 5);
        SetForegroundWindow(h);
    }
}
fn open_settings(hwnd: H) {
    let _dpi_context = DpiContext::new(-1);
    unsafe {
        let a = app(hwnd);
        if !a.settings.is_null() {
            ShowWindow(a.settings, 5);
            return;
        }
        let mut work = Rect::default();
        if SystemParametersInfoW(0x30, 0, (&mut work as *mut Rect).cast(), 0) == 0 {
            work.right = GetSystemMetrics(0);
            work.bottom = GetSystemMetrics(1);
        }
        let anchor = panel_anchor(a, hwnd);
        let anchor_id = a.extra_meters.iter().find(|m| m.hwnd == anchor).map(|m| m.id.as_str()).unwrap_or("");
        let displays = displays::list();
        let display = displays::selected(anchor_id).or_else(|| displays.iter().find(|d| d.primary).cloned());
        if let Some(display) = &display { work = Rect { left: display.work.left, top: display.work.top, right: display.work.right, bottom: display.work.bottom }; }
        let mut meter = Rect::default();
        if !anchor.is_null() { GetWindowRect(anchor, &mut meter); }
        let (x, y, width, height) = layout::settings_bounds(
            (meter.left, meter.top, meter.right, meter.bottom),
            (if work.right - work.left >= 1000 { 1000 } else { 520 }, 632),
            (work.left, work.top, work.right, work.bottom),
        );
        let h = CreateWindowExW(
            0x80,
            wide("MonitorPanel").as_ptr(),
            wide("Kurmon Settings").as_ptr(),
            0x80f80000,
            x, y, width, height,
            a.controller,
            null_mut(),
            GetModuleHandleW(std::ptr::null()),
            null_mut(),
        );
        SetWindowLongPtrW(h, -21, a as *mut App as isize);
        a.settings = h;
        fluent(h, is_light(a.config.theme));
        let heading = control(h, "STATIC", "Readings and sources", 0, 24, -20, 440, 24, 0);
        SendMessageW(heading, 0x30, a.heading as usize, 1);
        if settings_content_size(h).0 == 960 {
            let heading = control(h, "STATIC", "Taskbar display", 0, 504, -20, 440, 24, 0);
            SendMessageW(heading, 0x30, a.heading as usize, 1);
        }
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
        let all_gpus = control(h, "BUTTON", "Show all GPUs", 122, 260, 305, 210, 26, 0x10000 | 3);
        SendMessageW(all_gpus, 0xf1, a.config.all_gpus as usize, 0);
        for (i, (_id, name)) in a.gpus.iter().enumerate() {
            SendMessageW(gpu, 0x143, 0, wide(name).as_ptr() as isize);
            if name.eq_ignore_ascii_case(&a.config.gpu) {
                SendMessageW(gpu, 0x14e, i, 0);
            }
        }
        control(h, "STATIC", "Reading", 0, 24, 375, 100, 22, 0);
        let order_metric = control(h, "COMBOBOX", "", 127, 125, 370, 140, 160, 0x10000 | 3);
        for label in ["CPU", "GPU", "RAM", "Download", "Upload"] {
            SendMessageW(order_metric, 0x143, 0, wide(label).as_ptr() as isize);
        }
        SendMessageW(order_metric, 0x14e, 0, 0);
        control(h, "BUTTON", "Move up", 203, 275, 370, 90, 26, 0x10000 | 0xb);
        control(h, "BUTTON", "Move down", 204, 375, 370, 95, 26, 0x10000 | 0xb);
        control(h, "STATIC", "Order", 0, 24, 405, 55, 22, 0);
        control(
            h,
            "EDIT",
            &a.config.order,
            106,
            80,
            400,
            394,
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
        let displays: Vec<_> = displays::list().into_iter().filter(|display| !display.taskbar.is_null()).collect();
        let primary = SendMessageW(selector, 0x143, 0, wide("Primary display").as_ptr() as isize);
        let all = SendMessageW(selector, 0x143, 0, wide("All displays").as_ptr() as isize);
        let secondary = SendMessageW(selector, 0x143, 0, wide("Secondary displays only").as_ptr() as isize);
        if a.config.display.is_empty() || a.config.display == layout::PRIMARY_DISPLAY {
            SendMessageW(selector, 0x14e, primary as usize, 0);
        } else if a.config.display == layout::ALL_DISPLAYS {
            SendMessageW(selector, 0x14e, all as usize, 0);
        } else if a.config.display == layout::SECONDARY_DISPLAYS {
            SendMessageW(selector, 0x14e, secondary as usize, 0);
        }
        for display in &displays {
            let label = format!("{}{}", display.id, if display.primary { " (Primary)" } else { "" });
            let item = SendMessageW(selector, 0x143, 0, wide(&label).as_ptr() as isize);
            if display.id == a.config.display { SendMessageW(selector, 0x14e, item as usize, 0); }
        }
        control(h, "STATIC", "Meter font", 0, 24, 515, 90, 22, 0);
        let fonts = installed_fonts(h);
        let font = control(h, "COMBOBOX", "", 120, 120, 510, 220, 250, 0x10000 | 3 | 0x200000);
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
        control(h, "STATIC", "Size", 0, 345, 515, 40, 22, 0);
        let size = control(h, "COMBOBOX", "", 121, 390, 510, 80, 250, 0x10000 | 3);
        SendMessageW(size, 0x143, 0, wide("Auto").as_ptr() as isize);
        for pixels in 8..=32 {
            SendMessageW(size, 0x143, 0, wide(&pixels.to_string()).as_ptr() as isize);
        }
        SendMessageW(size, 0x14e, if a.config.font_size == 0 { 0 } else { (a.config.font_size - 7) as usize }, 0);
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
        let temperature = control(h, "BUTTON", "GPU temperature", 123, 260, 585, 210, 26, 0x10000 | 3);
        SendMessageW(temperature, 0xf1, a.config.gpu_temperature as usize, 0);
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
        let cpu_speed = control(h, "BUTTON", "CPU frequency", 124, 270, 620, 200, 26, 0x10000 | 3);
        SendMessageW(cpu_speed, 0xf1, a.config.cpu_speed as usize, 0);
        control(h, "STATIC", "Meter color", 0, 24, 655, 90, 22, 0);
        let palette = control(h, "COMBOBOX", "", 115, 120, 650, 165, 130, 0x10000 | 3);
        for label in ["Blue", "Green", "Amber"] {
            SendMessageW(palette, 0x143, 0, wide(label).as_ptr() as isize);
        }
        SendMessageW(palette, 0x14e, a.config.palette as usize, 0);
        control(h, "STATIC", "Columns", 0, 295, 655, 75, 22, 0);
        let columns = control(h, "COMBOBOX", "", 126, 370, 650, 100, 220, 0x10000 | 3);
        for label in ["Auto", "1", "2", "3", "4", "5", "6", "7", "8"] {
            SendMessageW(columns, 0x143, 0, wide(label).as_ptr() as isize);
        }
        SendMessageW(columns, 0x14e, a.config.grid_columns as usize, 0);
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
        let wide_layout = settings_content_size(h).0 == 960;
        for (i, label) in ["CPU chart", "GPU chart", "RAM chart", "Download chart", "Upload chart"].iter().enumerate() {
            let (x, y, width) = if wide_layout {
                (24 + (i as i32 % 2) * 220, 840 + (i as i32 / 2) * 30, 210)
            } else {
                (24 + (i as i32 % 3) * 145, 840 + (i as i32 / 3) * 30, 140)
            };
            let check = control(h, "BUTTON", label, 150 + i, x, y, width, 26, 0x10000 | 3);
            SendMessageW(check, 0xf1, a.config.charts[i] as usize, 0);
        }
        let (disk_x, disk_y, disk_width) = if wide_layout { (244, 900, 210) } else { (314, 870, 160) };
        let disk = control(h, "BUTTON", "Disk read/write", 125, disk_x, disk_y, disk_width, 26, 0x10000 | 3);
        SendMessageW(disk, 0xf1, a.config.disk_activity as usize, 0);
        control(h, "BUTTON", "Save settings", 201, 24, 910, 160, 36, 0x10000 | 0xb);
        control(h, "BUTTON", "Cancel", 202, 200, 910, 110, 36, 0x10000 | 0xb);
        let mut client = Rect::default();
        GetClientRect(h, &mut client);
        let (content_width, content_height) = settings_content_size(h);
        SetScrollRange(h, 0, 0, (content_width - client.right).max(0), 1);
        SetScrollRange(h, 1, 0, (content_height - client.bottom).max(0), 1);
        GetClientRect(h, &mut client);
        SetScrollRange(h, 0, 0, (content_width - client.right).max(0), 1);
        SetScrollRange(h, 1, 0, (content_height - client.bottom).max(0), 1);
        a.settings_original = Some(a.config.clone());
        a.settings_dirty = false;
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
unsafe fn scroll_settings_axis(hwnd: H, axis: i32, target: i32) {
    unsafe {
        let mut client = Rect::default();
        GetClientRect(hwnd, &mut client);
        let (width, height) = if hwnd == app(hwnd).details { (430, details_content_height(&app(hwnd).config, app(hwnd).gpus.len())) } else { settings_content_size(hwnd) };
        let max = if axis == 0 { width - client.right } else { height - client.bottom };
        let old = GetScrollPos(hwnd, axis);
        let target = target.clamp(0, max.max(0));
        if target != old {
            SetScrollPos(hwnd, axis, target, 1);
            let delta = old - target;
            ScrollWindowEx(hwnd, if axis == 0 { delta } else { 0 }, if axis == 1 { delta } else { 0 }, std::ptr::null(), std::ptr::null(), null_mut(), null_mut(), 3);
        }
    }
}

unsafe fn scroll_settings(hwnd: H, target: i32) {
    unsafe { scroll_settings_axis(hwnd, 1, target); }
}
fn preview_candidate(mut next: settings::Settings, current: &settings::Settings) -> (settings::Settings, bool) {
    let invalid = settings::Settings::parse(&next.serialize()).is_err();
    if invalid {
        if !next.visible.iter().any(|&visible| visible) {
            next.visible = current.visible;
        }
        if settings::Settings::parse(&next.serialize()).is_err() {
            next.order = current.order.clone();
        }
    }
    (next, invalid)
}

unsafe fn apply_settings(a: &mut App, next: settings::Settings) {
    unsafe {
        let worker_changed = a.config.adapter != next.adapter || a.config.gpu != next.gpu
            || a.config.gpu_temperature != next.gpu_temperature || a.config.interval != next.interval;
        a.config = next;
        if !a.details.is_null() {
            position_details(a, a.details, a.details_owner);
            fluent(a.details, is_light(a.config.theme));
            InvalidateRect(a.details, std::ptr::null(), 1);
        }
        if !a.settings.is_null() {
            fluent(a.settings, is_light(a.config.theme));
            InvalidateRect(a.settings, std::ptr::null(), 1);
        }
        if !a.meter.is_null() { meter_transparency(a.meter, a.config.transparent, a.config.theme); }
        for meter in &a.extra_meters {
            meter_transparency(meter.hwnd, a.config.transparent, a.config.theme);
            InvalidateRect(meter.hwnd, std::ptr::null(), 0);
        }
        if worker_changed { a.worker.configure(a.config.clone()); }
        KillTimer(a.controller, 1);
        SetTimer(a.controller, 1, a.config.interval * 1000, null_mut());
        if !a.meter.is_null() { InvalidateRect(a.meter, std::ptr::null(), 0); }
        invalidate_geometry(a);
        place(a);
        reconcile_extra_meters(a);
    }
}

unsafe fn close_settings(hwnd: H) {
    unsafe {
        let a = app(hwnd);
        if a.settings_dirty {
            let answer = choice_dialog(a, hwnd, "Unsaved settings", "Save changes to settings?",
                &["Save", "Discard", "Cancel"], false);
            if answer == 1 {
                SendMessageW(hwnd, 0x111, 201, 0);
                return;
            }
            if answer != 2 { return; }
        }
        if let Some(original) = a.settings_original.take() {
            apply_settings(a, original);
        }
        a.settings_dirty = false;
        DestroyWindow(hwnd);
    }
}

unsafe extern "system" fn proc(hwnd: H, msg: u32, wp: usize, lp: isize) -> isize {
    unsafe {
        if GetWindowLongPtrW(hwnd, -21) == 0 {
            return DefWindowProcW(hwnd, msg, wp, lp);
        }
        match msg {
            0x114 | 0x115 if hwnd == app(hwnd).settings || hwnd == app(hwnd).details => {
                let axis = if msg == 0x114 { 0 } else { 1 };
                let pos = GetScrollPos(hwnd, axis);
                let mut client = Rect::default();
                GetClientRect(hwnd, &mut client);
                let page = if axis == 0 { client.right } else { client.bottom };
                let target = match wp & 0xffff {
                    0 => pos - 30, 1 => pos + 30,
                    2 => pos - page, 3 => pos + page,
                    4 | 5 => (wp >> 16) as i32,
                    6 => 0, 7 => i32::MAX, _ => pos,
                };
                scroll_settings_axis(hwnd, axis, target);
                0
            }
            0x20a if hwnd == app(hwnd).settings || hwnd == app(hwnd).details => {
                let delta = ((wp >> 16) as u16 as i16) as i32;
                scroll_settings(hwnd, GetScrollPos(hwnd, 1) - delta);
                0
            }
            0x83 | 0x85 if hwnd == app(hwnd).meter || app(hwnd).extra_meters.iter().any(|meter| meter.hwnd == hwnd) => 0,
            0x84 if hwnd == app(hwnd).meter || app(hwnd).extra_meters.iter().any(|meter| meter.hwnd == hwnd) => {
                let a = app(hwnd);
                if hwnd != a.meter { return 1; }
                if !a.parent.is_null() {
                    // A child in Explorer's tree cannot start a caption move.
                    return 1;
                }
                let mut r = Rect::default();
                GetWindowRect(hwnd, &mut r);
                let x = (lp as u32 as u16 as i16) as i32;
                let y = ((lp as u32 >> 16) as u16 as i16) as i32;
                if a.config.locked {
                    1
                } else if x >= r.left && x < r.left + 8 && y >= r.top + 6 && y < r.bottom - 6 {
                    2
                } else {
                    DefWindowProcW(hwnd, msg, wp, lp)
                }
            }
            0x21 if hwnd == app(hwnd).meter => 3,
            0x231 if hwnd == app(hwnd).meter => {
                let mut rect = Rect::default();
                if GetWindowRect(hwnd, &mut rect) != 0 {
                    app(hwnd).drag_start = Some(rect);
                }
                0
            }
            0x214 | 0x216 if hwnd == app(hwnd).meter => {
                let a = app(hwnd);
                let proposed = &mut *(lp as *mut Rect);
                let valid = a.scan_context.zip(a.geometry.as_ref()).and_then(|((_, bounds, _), geometry)| {
                    let width = proposed.right - proposed.left;
                    layout::meter_position(bounds, width, 3, proposed.left - bounds.0, &geometry.occupied)
                        .map(|(x, y, height)| (x, y, width, height))
                });
                if let Some((x, y, width, height)) = valid {
                    *proposed = Rect { left: x, top: y, right: x + width, bottom: y + height };
                } else {
                    GetWindowRect(hwnd, proposed);
                }
                1
            }
            0x232 if hwnd == app(hwnd).meter => {
                let a = app(hwnd);
                let mut r = Rect::default();
                if GetWindowRect(hwnd, &mut r) != 0 {
                    if let Some(start) = a.drag_start.take() {
                        if r.right - r.left != start.right - start.left {
                            a.config.width = scale_pixels(r.right - r.left, 96, a.meter_dpi).clamp(260, 1200);
                        }
                        if r.left != start.left {
                            if let Some(display) = displays::selected(&a.config.display) {
                                let mut bar = Rect::default();
                                if GetWindowRect(display.taskbar, &mut bar) != 0 {
                                    a.config.position = 3;
                                    a.config.offset = scale_pixels((r.left - bar.left).max(0), 96, a.meter_dpi);
                                }
                            }
                        }
                    }
                }
                if a.settings_original.is_none() {
                    let _ = settings::load().and_then(|_| settings::save(&a.config));
                }
                place(app(hwnd));
                InvalidateRect(app(hwnd).meter, std::ptr::null(), 0);
                0
            }
            0x8003 => {
                place(app(hwnd));
                reconcile_extra_meters(app(hwnd));
                0
            }
            0x8002 => {
                invalidate_geometry(app(hwnd));
                invalidate_extra_geometry(app(hwnd));
                SetTimer(hwnd, 2, 200, null_mut());
                0
            }
            0x113 => {
                if wp == 2 {
                    KillTimer(hwnd, 2);
                    place(app(hwnd));
                    reconcile_extra_meters(app(hwnd));
                    return 0;
                }
                if wp == 3 {
                    let a = app(hwnd);
                    place(a);
                    reconcile_extra_meters(a);
                    if (a.meter.is_null() || a.geometry.is_some()) && a.scan_pending.is_none()
                        && a.extra_meters.iter().all(|meter| meter.geometry.is_some()) {
                        KillTimer(hwnd, 3);
                        a.scan_timer = false;
                    }
                    return 0;
                }
                if wp == 4 {
                    app(hwnd).rejected = null_mut();
                    place(app(hwnd));
                    return 0;
                }
                let a = app(hwnd);
                if a.drag_start.is_none() {
                    place(a);
                    reconcile_extra_meters(a);
                }
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
                let changed = accept_snapshot(&a.config, &mut a.snapshot, next);
                if changed || (new_sample && a.config.charts.iter().any(|&v| v)) {
                    if a.visible && !a.blocked && !a.meter.is_null() { InvalidateRect(a.meter, std::ptr::null(), 0); }
                    for meter in &a.extra_meters {
                        if a.visible && !meter.blocked && !meter.hwnd.is_null() { InvalidateRect(meter.hwnd, std::ptr::null(), 0); }
                    }
                }
                if !a.details.is_null() {
                    InvalidateRect(a.details, std::ptr::null(), 0);
                }
                0
            }
            0x205 if hwnd == app(hwnd).meter || app(hwnd).extra_meters.iter().any(|meter| meter.hwnd == hwnd) => {
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
            0x202 if hwnd == app(hwnd).meter || hwnd == app(hwnd).controller || app(hwnd).extra_meters.iter().any(|meter| meter.hwnd == hwnd) => {
                details(hwnd);
                0
            }
            0x111 => {
                let raw_id = wp & 0xffff;
                let notification = (wp >> 16) & 0xffff;
                let a = app(hwnd);
                if hwnd == a.choice && (301..=305).contains(&raw_id) && notification == 0 {
                    a.choice_result = (raw_id - 300) as i32;
                    DestroyWindow(hwnd);
                    return 0;
                }
                let preview = hwnd == a.settings && a.settings_original.is_some()
                    && ((matches!(raw_id, 101..=105 | 108..=109 | 115 | 120..=121 | 126 | 135) && notification == 1)
                        || (matches!(raw_id, 104 | 107 | 112 | 114 | 122..=125 | 130..=134 | 140..=144 | 150..=154) && notification == 0)
                        || (raw_id == 106 && notification == 0x300));
                if preview { a.settings_dirty = true; }
                let id = if preview { 201 } else { raw_id };
                match id {
                    501 => open_settings(a.controller),
                    502 => {
                        let _ = std::process::Command::new("taskmgr.exe").spawn();
                    }
                    203 | 204 if hwnd == a.settings => {
                        let selected = SendMessageW(get_child(hwnd, 127), 0x147, 0, 0);
                        if let Some(name) = ["CPU", "GPU", "RAM", "Download", "Upload"].get(selected as usize) {
                            let mut buffer = [0u16; 128];
                            let count = GetWindowTextW(get_child(hwnd, 106), buffer.as_mut_ptr(), 128);
                            let order = String::from_utf16_lossy(&buffer[..count as usize]);
                            let mut parts: Vec<_> = order.split(',').collect();
                            if let Some(index) = parts.iter().position(|part| part == name) {
                                let target = if id == 203 { index.saturating_sub(1) } else { (index + 1).min(parts.len() - 1) };
                                parts.swap(index, target);
                                SetWindowTextW(get_child(hwnd, 106), wide(&parts.join(",")).as_ptr());
                            }
                        }
                    }
                    202 => {
                        close_settings(hwnd);
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
                        next.all_gpus = SendMessageW(get_child(hwnd, 122), 0xf0, 0, 0) == 1;
                        next.gpu_temperature = SendMessageW(get_child(hwnd, 123), 0xf0, 0, 0) == 1;
                        next.cpu_speed = SendMessageW(get_child(hwnd, 124), 0xf0, 0, 0) == 1;
                        next.disk_activity = SendMessageW(get_child(hwnd, 125), 0xf0, 0, 0) == 1;
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
                        let displays: Vec<_> = displays::list().into_iter().filter(|display| !display.taskbar.is_null()).collect();
                        next.display = match index {
                            0 if a.settings_original.as_ref().is_some_and(|s| s.display.is_empty()) => String::new(),
                            0 => layout::PRIMARY_DISPLAY.into(),
                            1 => layout::ALL_DISPLAYS.into(),
                            2 => layout::SECONDARY_DISPLAYS.into(),
                            i if i >= 3 => displays.get((i - 3) as usize).map(|d| d.id.clone()).unwrap_or_else(|| layout::PRIMARY_DISPLAY.into()),
                            _ => layout::PRIMARY_DISPLAY.into(),
                        };
                        let position_index = SendMessageW(get_child(hwnd, 109), 0x147, 0, 0) as i32;
                        next.position = position_index - 1;
                        if next.position != 3 {
                            next.offset = -1;
                        }
                        else if let Some(original) = &a.settings_original {
                            if original.position == 3 && a.config.position != 3 {
                                next.offset = original.offset;
                            }
                        }
                        next.locked = SendMessageW(get_child(hwnd, 112), 0xf0, 0, 0) == 1;
                        let mut face = [0u16; 128];
                        let count = GetWindowTextW(get_child(hwnd, 120), face.as_mut_ptr(), 128);
                        if count > 0 {
                            next.font = String::from_utf16_lossy(&face[..count as usize]);
                        }
                        let size = SendMessageW(get_child(hwnd, 121), 0x147, 0, 0);
                        next.font_size = if size <= 0 { 0 } else { size as u32 + 7 };
                        next.transparent = SendMessageW(get_child(hwnd, 114), 0xf0, 0, 0) == 1;
                        next.palette =
                            SendMessageW(get_child(hwnd, 115), 0x147, 0, 0).clamp(0, 2) as u32;
                        next.grid_columns = SendMessageW(get_child(hwnd, 126), 0x147, 0, 0).clamp(0, 8) as u32;
                        let (next, invalid_preview) = if preview {
                            preview_candidate(next, &a.config)
                        } else {
                            (next, false)
                        };
                        let result = settings::Settings::parse(&next.serialize())
                            .and_then(|_| if preview {
                                Ok(())
                            } else if next.startup == a.settings_original.as_ref().map(|s| s.startup).unwrap_or(a.config.startup) {
                                settings::load().and_then(|_| settings::save(&next))
                            } else {
                                save_with_startup(&next)
                            });
                        match result {
                            Ok(()) => {
                                if preview {
                                    let original = a.settings_original.as_ref().unwrap();
                                    a.settings_dirty = invalid_preview || next.serialize() != original.serialize();
                                }
                                apply_settings(a, next);
                                if !preview {
                                    a.settings_original = None;
                                    a.settings_dirty = false;
                                    DestroyWindow(hwnd);
                                }
                            }
                            Err(e) => {
                                if preview { return 0; }
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
                if hwnd == a.settings {
                    #[link(name = "gdi32")]
                    unsafe extern "system" {
                        fn RoundRect(dc: H, left: i32, top: i32, right: i32, bottom: i32, w: i32, h: i32) -> i32;
                    }
                    let card = CreateSolidBrush(if light { 0xffffff } else { 0x232323 });
                    let border = CreatePen(0, 1, if light { 0xd8d8d8 } else { 0x353535 });
                    let old_brush = SelectObject(dc, card);
                    let old_pen = SelectObject(dc, border);
                    let dx = GetScrollPos(hwnd, 0);
                    let dy = GetScrollPos(hwnd, 1);
                    let wide = settings_content_size(hwnd).0 == 960;
                    let bottom = if wide { 520 } else { 938 };
                    RoundRect(dc, 8 - dx, 4 - dy, 480 - dx, bottom - dy, 10, 10);
                    if wide { RoundRect(dc, 488 - dx, 4 - dy, 960 - dx, bottom - dy, 10, 10); }
                    SelectObject(dc, old_brush);
                    SelectObject(dc, old_pen);
                    DeleteObject(card);
                    DeleteObject(border);
                }
                else if hwnd == a.details {
                    #[link(name = "gdi32")]
                    unsafe extern "system" {
                        fn RoundRect(dc: H, left: i32, top: i32, right: i32, bottom: i32, w: i32, h: i32) -> i32;
                    }
                    let surface = CreateSolidBrush(if light { 0xffffff } else { 0x262626 });
                    let border = CreatePen(0, 1, if light { 0xdcdcdc } else { 0x383838 });
                    let old_brush = SelectObject(dc, surface);
                    let old_pen = SelectObject(dc, border);
                    let dy = GetScrollPos(hwnd, 1);
                    RoundRect(dc, 12, 40 - dy, 430, 186 - dy, 10, 10);
                    RoundRect(dc, 12, 190 - dy, 430, 264 + details_extra_height(&a.config, a.gpus.len()) - dy, 10, 10);
                    SelectObject(dc, old_brush);
                    SelectObject(dc, old_pen);
                    DeleteObject(surface);
                    DeleteObject(border);
                }
                SetTextColor(dc, fg);
                SetBkMode(dc, 1);
                SelectObject(dc, a.small);
                let s = &a.snapshot;
                if hwnd == a.meter || a.extra_meters.iter().any(|meter| meter.hwnd == hwnd) {
                    // Fixed slots stop digits from shifting the neighbouring readings.
                    let selected = selected_metrics(&a.config, &a.gpus);
                    let mut client = Rect::default();
                    GetClientRect(hwnd, &mut client);
                    let meter_dpi = a.extra_meters.iter().find(|meter| meter.hwnd == hwnd).map(|meter| meter.dpi).unwrap_or(a.meter_dpi);
                    let meter_layout = layout::responsive_meter(client.right, client.bottom, meter_dpi, &selected, a.config.ram_format == 2, a.config.grid_columns);
                    let muted = if light { rgb(0x55, 0x55, 0x55) } else { rgb(0xb8, 0xb8, 0xb8) };
                    let accent = match (a.config.palette, light) {
                        (1, false) => rgb(0x6c, 0xcb, 0x8a),
                        (1, true) => rgb(0x18, 0x80, 0x38),
                        (2, false) => rgb(0xf0, 0xb4, 0x4d),
                        (2, true) => rgb(0x9d, 0x5d, 0x00),
                        (_, false) => rgb(0x4c, 0xc2, 0xff),
                        (_, true) => rgb(0x0f, 0x6c, 0xbd),
                    };
                    let readings: Vec<_> = selected.iter().map(|&i| {
                        let (label, mut value) = match i {
                            0 => ("CPU ".to_string(), percentage(s.cpu)),
                            1 => (if a.config.all_gpus { format!("GPU{} ", a.gpus.iter().position(|(id, _)| id.eq_ignore_ascii_case(&s.gpu_id)).unwrap_or(0) + 1) } else { "GPU ".to_string() }, percentage(s.gpu)),
                            2 => ("RAM ".to_string(), ram_value(s.ram, s.used, a.config.ram_format)),
                            3 => ("DL ".to_string(), rate(s.download, a.config.bits)),
                            4 => ("UL ".to_string(), rate(s.upload, a.config.bits)),
                            CPU_CLOCK => ("Clock ".to_string(), cpu_clock(s.cpu_mhz)),
                            DISK_READ => ("Disk R ".to_string(), rate(s.disk_read, false)),
                            DISK_WRITE => ("Disk W ".to_string(), rate(s.disk_write, false)),
                            _ => (format!("GPU{} ", i - 4), percentage(s.gpus.iter().find(|(id, _)| id.eq_ignore_ascii_case(&a.gpus[i - 5].0)).and_then(|(_, value)| *value))),
                        };
                        if a.config.gpu_temperature && (i == 1 || (i >= 5 && i < DISK_WRITE)) {
                            let id = if i == 1 { s.gpu_id.as_str() } else { a.gpus[i - 5].0.as_str() };
                            let temp = s.gpu_temperatures.iter().find(|(gpu, _)| gpu.eq_ignore_ascii_case(id)).map(|(_, t)| format!("{t:.0}°C")).unwrap_or_else(|| "--°C".into());
                            value.push_str(&format!(" {temp}"));
                        }
                        let show_label = match i {
                            CPU_CLOCK => a.config.labels[0],
                            DISK_READ | DISK_WRITE => true,
                            5.. => a.config.labels[1],
                            _ => a.config.labels[i],
                        };
                        (if show_label { label } else { String::new() }, value)
                    }).collect();
                    let face = wide(&a.config.font);
                    let mut font_px = if a.config.font_size == 0 { meter_layout.font_px } else { scale_pixels(a.config.font_size as i32, meter_dpi, 96) }.max(1);
                    let fitted_font = loop {
                        let font = CreateFontW(-font_px, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, face.as_ptr());
                        if font.is_null() { break font; }
                        SelectObject(dc, font);
                        let fits = selected.iter().enumerate().all(|(slot, &i)| {
                            let (left, top, right, bottom) = meter_layout.cells[slot];
                            let icon = if readings[slot].0.is_empty() { 0 } else { scale_pixels(18, meter_dpi, 96) };
                            let chart = if i < 5 && a.config.charts[i] { scale_pixels(18, meter_dpi, 96).min((right - left) / 3) } else { 0 };
                            let text = wide(&format!("{}{}", readings[slot].0, readings[slot].1));
                            let mut extent = [0; 2];
                            GetTextExtentPoint32W(dc, text.as_ptr(), (text.len() - 1) as i32, &mut extent) != 0
                                && extent[0] + icon + scale_pixels(8, meter_dpi, 96) <= right - left - chart && extent[1] <= bottom - top
                        });
                        if fits || font_px == 1 { break font; }
                        SelectObject(dc, a.small);
                        DeleteObject(font);
                        font_px -= 1;
                    };
                    if !fitted_font.is_null() {
                    let value_font = CreateFontW(-font_px, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0, face.as_ptr());
                    for (slot, &i) in selected.iter().enumerate() {
                        let (left, top, right, bottom) = meter_layout.cells[slot];
                        if right <= left || bottom <= top || meter_layout.font_px <= 0 {
                            continue;
                        }
                        let saved = SaveDC(dc);
                        if saved == 0 {
                            continue;
                        }
                        IntersectClipRect(dc, left, top, right, bottom);
                        let (prefix, value) = &readings[slot];
                        let prefix = prefix.as_str();
                        let chart_width = if i < 5 && a.config.charts[i] { scale_pixels(18, meter_dpi, 96).min((right - left) / 3) } else { 0 };
                        let text_right = right - chart_width;
                        let used = if i >= 5 && i < DISK_WRITE {
                            s.gpus.iter().find(|(id, _)| id.eq_ignore_ascii_case(&a.gpus[i - 5].0)).and_then(|(_, value)| *value)
                        } else {
                            [s.cpu, s.gpu, s.ram].get(i).copied().flatten()
                        };
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
                        let icon_width = if prefix.is_empty() {
                            0
                        } else {
                            draw_metric_icon(dc, i, Rect { left, top, right: text_right, bottom }, accent, meter_dpi)
                        };
                        let text_left = left + icon_width;
                        let mut cell = Rect {
                            left: text_left,
                            top,
                            right: text_right,
                            bottom,
                        };
                        SetTextColor(dc, muted);
                        DrawTextW(dc, wide(prefix).as_ptr(), -1, &mut cell, 0x824);
                        let mut size = [0i32; 2];
                        GetTextExtentPoint32W(
                            dc,
                            wide(prefix).as_ptr(),
                            prefix.encode_utf16().count() as i32,
                            &mut size,
                        );
                        let mut cell = Rect {
                            left: (text_left + size[0]).min(text_right),
                            top,
                            right: text_right,
                            bottom,
                        };
                        if !value_font.is_null() {
                            SelectObject(dc, value_font);
                        }
                        SetTextColor(dc, value_color);
                        DrawTextW(dc, wide(&value).as_ptr(), -1, &mut cell, 0x824 | 0x8000);
                        SelectObject(dc, fitted_font);
                        if chart_width > 0 {
                            let chart_left = text_right + 1;
                            let inset = scale_pixels(2, meter_dpi, 96);
                            if right - chart_left >= 2 && bottom - top > inset * 2 {
                                draw_history(dc, &a.history, i, Rect { left: chart_left, top: top + inset, right, bottom: bottom - inset }, accent);
                            }
                        }
                        RestoreDC(dc, saved);
                    }
                    SelectObject(dc, a.small);
                    DeleteObject(fitted_font);
                    if !value_font.is_null() { DeleteObject(value_font); }
                    }
                } else if hwnd == a.details {
                    let name = a
                        .devices
                        .iter()
                        .find(|d| d.id == a.config.adapter)
                        .map(|d| d.name.as_str())
                        .unwrap_or("Select adapter");
                    let values = [
                        percentage(s.cpu),
                        percentage(s.gpu),
                        format!("{}  ({:.1} / {:.1} GiB)", percentage(s.ram), s.used as f64 / 1073741824.0, s.total as f64 / 1073741824.0),
                        rate(s.download, a.config.bits),
                        rate(s.upload, a.config.bits),
                    ];
                    for (i, value) in values.iter().enumerate() {
                        SetWindowTextW(get_child(hwnd, 610 + i as i32), wide(value).as_ptr());
                    }
                    let notice = if a.blocked { "Meter hidden: insufficient taskbar space" } else { a.notice.as_str() };
                    let gpu_lines = gpu_status_lines(&a.gpus, s);
                    let mut status = format!("{name} · {}", s.network_status);
                    if gpu_lines.is_empty() {
                        status.push_str("\nGPU: unavailable");
                    } else {
                        for line in gpu_lines {
                            status.push('\n');
                            status.push_str(&line);
                        }
                    }
                    if a.config.cpu_speed { status.push_str(&format!("\nCPU frequency: {}", cpu_clock(s.cpu_mhz))); }
                    if a.config.disk_activity { status.push_str(&format!("\nDisk: read {} · write {}", rate(s.disk_read, false), rate(s.disk_write, false))); }
                    if !notice.is_empty() { status.push_str(&format!("\n{notice}")); }
                    SetWindowTextW(get_child(hwnd, 620), wide(&status).as_ptr());
                    let dx = GetScrollPos(hwnd, 0);
                    let dy = GetScrollPos(hwnd, 1);
                    let names = ["CPU", "GPU", "RAM", "Download", "Upload"];
                    let mut chart_offset = 0;
                    for i in 0..5 {
                        if a.config.visible[i] && a.config.charts[i] {
                            let top = 265 + details_extra_height(&a.config, a.gpus.len()) + chart_offset - dy;
                            let mut label = Rect { left: 24 - dx, top, right: 115 - dx, bottom: top + 24 };
                            SetTextColor(dc, fg);
                            DrawTextW(dc, wide(names[i]).as_ptr(), -1, &mut label, 0x824);
                            if i == 2 {
                                draw_ram_history(dc, &a.history, Rect { left: 24 - dx, top: top + 24, right: 410 - dx, bottom: top + 132 }, light);
                                chart_offset += 140;
                            } else {
                                draw_history(dc, &a.history, i, Rect { left: 120 - dx, top, right: 410 - dx, bottom: top + 30 }, if light { rgb(0x0f, 0x6c, 0xbd) } else { rgb(0x4c, 0xc2, 0xff) });
                                chart_offset += 40;
                            }
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
                if hwnd == app(hwnd).settings {
                    close_settings(hwnd);
                } else if hwnd != app(hwnd).meter {
                    DestroyWindow(hwnd);
                }
                0
            }
            0x6 => {
                let a = app(hwnd);
                if hwnd == a.choice && a.choice_menu && wp & 0xffff == 0 {
                    DestroyWindow(hwnd);
                    return 0;
                }
                if wp & 0xffff == 0 && hwnd == a.details && lp as H != a.settings {
                    ShowWindow(hwnd, 0);
                }
                0
            }
            0x10 => {
                if hwnd == app(hwnd).settings { close_settings(hwnd); } else { DestroyWindow(hwnd); }
                0
            }
            0x2 => {
                let a = app(hwnd);
                if hwnd == a.choice {
                    a.choice = null_mut();
                    return 0;
                }
                if hwnd == a.controller {
                    if !a.meter.is_null() {
                        DestroyWindow(a.meter);
                        a.meter = null_mut();
                    }
                    for meter in a.extra_meters.drain(..) { DestroyWindow(meter.hwnd); }
                    notify(hwnd, true);
                    PostQuitMessage(0);
                } else if hwnd == a.meter {
                    a.meter = null_mut();
                } else if let Some(index) = a.extra_meters.iter().position(|meter| meter.hwnd == hwnd) {
                    a.extra_meters.remove(index);
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
                if msg == 0x138 && hwnd == app(hwnd).details {
                    SetBkMode(wp as H, 1);
                    return GetStockObject(5) as isize;
                }
                let color = if hwnd == app(hwnd).settings {
                    if light { 0xffffff } else { 0x292929 }
                } else if light { 0xf5f5f5 } else { 0x202020 };
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
            0x7e => {
                let a = app(hwnd);
                invalidate_geometry(a);
                invalidate_extra_geometry(a);
                SetTimer(hwnd, 2, 200, null_mut());
                0
            }
            0x2e0 => {
                let controller = app(hwnd).controller;
                PostMessageW(controller, 0x8002, 0, 0);
                invalidate_extra_geometry(app(controller));
                0
            }
            0x1a => {
                let a = app(hwnd);
                invalidate_geometry(a);
                invalidate_extra_geometry(a);
                0
            }
            _ => {
                if msg == RegisterWindowMessageW(w!("TaskbarCreated").as_ptr()) {
                    let a = app(hwnd);
                    invalidate_geometry(a);
                    invalidate_extra_geometry(a);
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
        if SetProcessDpiAwarenessContext(-4isize as H) == 0
            && SetProcessDpiAwarenessContext(-3isize as H) == 0
        {
            SetProcessDPIAware();
        }
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
        for name in ["TaskbarSystemMonitor", "MonitorPanel", "KurmonMeter", "KurmonHit"] {
            let class_name = wide(name);
            let class = Class {
                style: 0,
                proc: Some(if name == "KurmonHit" { hit_proc } else { proc }),
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
        let heading = CreateFontW(-18, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0, w!("Segoe UI").as_ptr());
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
            heading,
            details: null_mut(),
            details_owner: null_mut(),
            settings: null_mut(),
            settings_original: None,
            settings_dirty: false,
            choice: null_mut(),
            choice_result: 0,
            choice_menu: false,
            controller: null_mut(),
            meter: null_mut(),
            parent: null_mut(),
            rejected: null_mut(),
            retries: 0,
            scan: scan::Scanner::new(),
            geometry: None,
            scan_generation: 0,
            scan_context: None,
            scan_pending: None,
            scan_timer: false,
            visible: true,
            meter_dpi: 96,
            drag_start: None,
            blocked: false,
            extra_meters: Vec::new(),
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
        reconcile_extra_meters(&mut a);
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
                    SendMessageW(panel, 0x10, 0, 0);
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
                let _dpi_context = DpiContext::new(-1);
                let focus = GetFocus();
                let panel = if !a.settings.is_null() && IsChild(a.settings, focus) != 0 {
                    a.settings
                } else if !a.details.is_null() && IsChild(a.details, focus) != 0 {
                    a.details
                } else {
                    null_mut()
                };
                if !panel.is_null() {
                    let mut rect = Rect::default();
                    let mut client = Rect::default();
                    GetWindowRect(focus, &mut rect);
                    GetClientRect(panel, &mut client);
                    let mut top = [rect.left, rect.top];
                    let mut bottom = [rect.right, rect.bottom];
                    ScreenToClient(panel, &mut top);
                    ScreenToClient(panel, &mut bottom);
                    let pos = GetScrollPos(panel, 1);
                    if top[1] < 0 {
                        scroll_settings(panel, pos + top[1]);
                    } else if bottom[1] > client.bottom {
                        scroll_settings(panel, pos + bottom[1] - client.bottom);
                    }
                    let pos = GetScrollPos(panel, 0);
                    if top[0] < 0 {
                        scroll_settings_axis(panel, 0, pos + top[0]);
                    } else if bottom[0] > client.right {
                        scroll_settings_axis(panel, 0, pos + bottom[0] - client.right);
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
        if !a.heading.is_null() { DeleteObject(a.heading); }
        CloseHandle(mutex);
        CoUninitialize();
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::commit_startup;
    use std::cell::RefCell;

    #[test]
    fn invalid_edit_preserves_valid_preview_changes() {
        let original = super::settings::Settings::default();
        let mut next = original.clone();
        next.order = "CPU,".into();
        next.palette = 2;
        let (preview, invalid) = super::preview_candidate(next, &original);
        assert!(invalid);
        assert_eq!(preview.order, original.order);
        assert_eq!(preview.palette, 2);
    }

    #[test]
    fn all_gpus_keeps_distinct_adapters_with_identical_names() {
        let mut settings = super::settings::Settings::default();
        settings.gpu = "GPU".into();
        settings.all_gpus = true;
        let gpus = vec![("first".into(), "GPU".into()), ("second".into(), "GPU".into())];
        assert_eq!(super::selected_metrics(&settings, &gpus), [0, 1, 6, 2, 3, 4]);
        settings.visible[1] = false;
        assert_eq!(super::selected_metrics(&settings, &gpus), [0, 2, 3, 4]);
    }

    #[test]
    fn logical_geometry_round_trips_at_common_dpi() {
        for dpi in [96, 120, 144, 192] {
            for logical in [120, 260, 500, 1200] {
                let physical = super::scale_pixels(logical, dpi, 96);
                assert_eq!(super::scale_pixels(physical, 96, dpi), logical);
            }
        }
    }

    #[test]
    fn failed_settings_save_restores_exact_startup_value() {
        let registry = RefCell::new(Some((7, vec![1, 2, 3])));
        let before = registry.borrow().clone();
        let result = commit_startup(
            before.clone(),
            || { *registry.borrow_mut() = None; Ok(()) },
            || Err("disk failure".into()),
            |value| { *registry.borrow_mut() = value; Ok(()) },
        );
        assert!(result.is_err());
        assert_eq!(*registry.borrow(), before);
    }

    #[test]
    fn failed_registry_update_leaves_settings_untouched_and_restores_absence() {
        let registry = RefCell::new(None);
        let saved = RefCell::new(false);
        let result = commit_startup(
            None,
            || { *registry.borrow_mut() = Some(9); Err("registry failure".into()) },
            || { *saved.borrow_mut() = true; Ok(()) },
            |value| { *registry.borrow_mut() = value; Ok(()) },
        );
        assert!(result.is_err());
        assert_eq!(*registry.borrow(), None);
        assert!(!*saved.borrow());
    }

    #[test]
    fn rollback_failure_is_reported() {
        let result = commit_startup(
            (),
            || Ok(()),
            || Err("disk failure".into()),
            |_| Err("registry rollback failure".into()),
        );
        let error = result.unwrap_err();
        assert!(error.contains("disk failure"));
        assert!(error.contains("registry rollback failure"));
    }


    #[test]
    fn ram_percent_and_used_have_a_separator() {
        assert_eq!(super::ram_value(Some(66.0), 10_737_418_240, 2), "66% · 10.0 GiB");
        assert_eq!(super::ram_value(None, 0, 2), "--");
    }

    #[test]
    fn accepted_worker_sample_replaces_displayed_snapshot() {
        let settings = super::settings::Settings::default();
        let mut current = super::sampler::Snapshot::default();
        let mut next = super::sampler::Snapshot::default();
        next.cpu = Some(42.0);
        next.ram = Some(73.0);
        assert!(super::accept_snapshot(&settings, &mut current, next));
        assert_eq!(current.cpu, Some(42.0));
        assert_eq!(current.ram, Some(73.0));
    }

    #[test]
    fn overview_lists_each_gpu_name_with_its_live_value() {
        let gpus = vec![
            ("gpu-a".into(), "GPU A".into()),
            ("gpu-b".into(), "GPU B".into()),
        ];
        let mut snapshot = super::sampler::Snapshot::default();
        snapshot.gpus = vec![("gpu-a".into(), Some(12.0)), ("gpu-b".into(), Some(34.0))];
        assert_eq!(
            super::gpu_status_lines(&gpus, &snapshot),
            [
                "GPU 1 · 12% · GPU A",
                "GPU 2 · 34% · GPU B",
            ]
        );
    }

    #[test]
    fn ram_chart_gets_extra_space_only_when_enabled_and_visible() {
        let mut config = super::settings::Settings::default();
        let baseline = super::details_content_height(&config, 2);
        config.charts[2] = true;
        assert_eq!(super::details_content_height(&config, 2) - baseline, 140);
        config.visible[2] = false;
        assert_eq!(super::details_content_height(&config, 2), baseline);
    }
}
