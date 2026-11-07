// UI Automation must not run on a thread that owns windows: our meter is a
// descendant of the scanned taskbar, so a synchronous scan can deadlock.
use crate::displays;
use std::{
    ffi::c_void,
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
};
use windows::Win32::{Foundation::HWND, System::Com::*, UI::Accessibility::*};

#[derive(Clone)]
pub struct Layout {
    pub occupied: Vec<(i32, i32)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Request {
    pub bar: usize,
    pub generation: u64,
    pub bounds: (i32, i32, i32, i32),
}

pub struct ScanResult {
    pub request: Request,
    pub occupied: Option<Vec<(i32, i32)>>,
}

impl ScanResult {
    pub fn layout(self, generation: u64, bar: usize, bounds: (i32, i32, i32, i32)) -> Option<Layout> {
        if self.request != (Request { bar, generation, bounds }) {
            return None;
        }
        self.occupied.map(|occupied| Layout { occupied })
    }
}

pub struct Scanner {
    request: SyncSender<Request>,
    results: Receiver<ScanResult>,
}

impl Scanner {
    pub fn new() -> Self {
        let (request, requests) = mpsc::sync_channel::<Request>(1);
        let (results_tx, results) = mpsc::channel::<ScanResult>();
        thread::spawn(move || {
            let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).is_ok() };
            while let Ok(request) = requests.recv() {
                let occupied = if initialized { scan(request) } else { None };
                if results_tx.send(ScanResult { request, occupied }).is_err() {
                    break;
                }
            }
            if initialized {
                unsafe { CoUninitialize() };
            }
        });
        Self { request, results }
    }
    pub fn request(&self, request: Request) -> bool {
        self.request.try_send(request).is_ok()
    }
    pub fn latest(&self) -> Option<ScanResult> {
        self.results.try_recv().ok()
    }
}

fn scan(request: Request) -> Option<Vec<(i32, i32)>> {
    let bar = request.bar as *mut c_void;
    if bar.is_null() || bar_bounds(bar)? != request.bounds {
        return None;
    }
    let occupied = collect(bar, request.bounds).ok()?;
    (bar_bounds(bar)? == request.bounds).then_some(occupied)
}

fn bar_bounds(bar: *mut c_void) -> Option<(i32, i32, i32, i32)> {
    let mut rect = displays::Rect::default();
    unsafe {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn GetWindowRect(hwnd: *mut c_void, rect: *mut displays::Rect) -> i32;
        }
        if GetWindowRect(bar, &mut rect) == 0 {
            return None;
        }
    }
    Some((rect.left, rect.top, rect.right, rect.bottom))
}

fn collect(
    bar: *mut c_void,
    bounds: (i32, i32, i32, i32),
) -> windows::core::Result<Vec<(i32, i32)>> {
    unsafe {
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
        let root = automation.ElementFromHandle(HWND(bar))?;
        let nodes = root.FindAll(TreeScope_Descendants, &automation.CreateTrueCondition()?)?;
        let mut occupied = vec![];
        for i in 0..nodes.Length()? {
            let element = nodes.GetElement(i)?;
            if element.CurrentControlType()? != UIA_ButtonControlTypeId {
                continue;
            }
            let rect = element.CurrentBoundingRectangle()?;
            let rect = (rect.left, rect.top, rect.right, rect.bottom);
            if crate::layout::intersects_taskbar(rect, bounds) {
                occupied.push((rect.0, rect.2));
            }
        }
        Ok(occupied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obsolete_and_failed_scans_do_not_supply_geometry() {
        let request = Request { bar: 7, generation: 3, bounds: (0, 0, 1000, 40) };
        let result = || ScanResult { request, occupied: Some(vec![(10, 20)]) };
        assert!(result().layout(4, 7, request.bounds).is_none());
        assert!(result().layout(3, 8, request.bounds).is_none());
        assert!(result().layout(3, 7, (0, 0, 900, 40)).is_none());
        assert_eq!(result().layout(3, 7, request.bounds).unwrap().occupied, vec![(10, 20)]);
        let failed = ScanResult { request, occupied: None };
        assert!(failed.layout(3, 7, request.bounds).is_none());
    }
}
