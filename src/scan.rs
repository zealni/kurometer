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
    pub bar: usize,
    pub occupied: Vec<(i32, i32)>,
}

pub struct Scanner {
    request: SyncSender<usize>,
    results: Receiver<Layout>,
}

impl Scanner {
    pub fn new() -> Self {
        let (request, requests) = mpsc::sync_channel::<usize>(1);
        let (results_tx, results) = mpsc::sync_channel::<Layout>(1);
        thread::spawn(move || {
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            }
            while let Ok(bar) = requests.recv() {
                if let Some(layout) = scan(bar as *mut c_void) {
                    let _ = results_tx.try_send(layout);
                }
            }
        });
        Self { request, results }
    }
    pub fn request(&self, bar: *mut c_void) {
        // A queued scan for another taskbar is re-requested by the next placement.
        let _ = self.request.try_send(bar as usize);
    }
    pub fn latest(&self) -> Option<Layout> {
        let mut latest = None;
        while let Ok(value) = self.results.try_recv() {
            latest = Some(value);
        }
        latest
    }
}

fn scan(bar: *mut c_void) -> Option<Layout> {
    if bar.is_null() {
        return None;
    }
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
    let bounds = (rect.left, rect.top, rect.right, rect.bottom);
    let occupied = collect(bar, bounds).ok()?;
    Some(Layout {
        bar: bar as usize,
        occupied,
    })
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
