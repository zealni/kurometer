use std::{ffi::{c_char, c_void, CStr}, ptr};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(name: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> i32;
}

struct Library(*mut c_void);

impl Library {
    fn system(name: &str) -> Option<Self> {
        let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let module = unsafe { LoadLibraryExW(name.as_ptr(), ptr::null_mut(), 0x800) };
        (!module.is_null()).then_some(Self(module))
    }

    fn symbol(&self, name: &CStr) -> *mut c_void {
        unsafe { GetProcAddress(self.0, name.as_ptr()) }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.0); }
    }
}

pub fn gpu_temperatures() -> Vec<(String, f64)> {
    unsafe {
        let Some(cuda) = Library::system("nvcuda.dll") else { return Vec::new() };
        let Some(nvml) = Library::system("nvml.dll") else { return Vec::new() };
        macro_rules! load {
            ($library:expr, $name:literal, $ty:ty) => {{
                let address = $library.symbol($name);
                if address.is_null() { return Vec::new(); }
                std::mem::transmute::<*mut c_void, $ty>(address)
            }};
        }
        let cu_init = load!(cuda, c"cuInit", unsafe extern "system" fn(u32) -> i32);
        let cu_count = load!(cuda, c"cuDeviceGetCount", unsafe extern "system" fn(*mut i32) -> i32);
        let cu_device = load!(cuda, c"cuDeviceGet", unsafe extern "system" fn(*mut i32, i32) -> i32);
        let cu_luid = load!(cuda, c"cuDeviceGetLuid", unsafe extern "system" fn(*mut u8, *mut u32, i32) -> i32);
        let cu_pci = load!(cuda, c"cuDeviceGetPCIBusId", unsafe extern "system" fn(*mut c_char, i32, i32) -> i32);
        let init = load!(nvml, c"nvmlInit_v2", unsafe extern "system" fn() -> i32);
        let shutdown = load!(nvml, c"nvmlShutdown", unsafe extern "system" fn() -> i32);
        let device_by_pci = load!(nvml, c"nvmlDeviceGetHandleByPciBusId_v2", unsafe extern "system" fn(*const c_char, *mut *mut c_void) -> i32);
        let temperature = load!(nvml, c"nvmlDeviceGetTemperature", unsafe extern "system" fn(*mut c_void, u32, *mut u32) -> i32);

        if cu_init(0) != 0 || init() != 0 { return Vec::new(); }
        let mut count = 0;
        let mut readings = Vec::new();
        if cu_count(&mut count) == 0 {
            for index in 0..count {
                let mut device = 0;
                let mut luid = [0u8; 8];
                let mut node_mask = 0;
                let mut pci = [0 as c_char; 32];
                let mut handle = ptr::null_mut();
                let mut value = 0;
                if cu_device(&mut device, index) != 0
                    || cu_luid(luid.as_mut_ptr(), &mut node_mask, device) != 0
                    || node_mask != 1
                    || cu_pci(pci.as_mut_ptr(), pci.len() as i32, device) != 0
                    || device_by_pci(pci.as_ptr(), &mut handle) != 0
                    || temperature(handle, 0, &mut value) != 0
                {
                    continue;
                }
                let low = u32::from_le_bytes([luid[0], luid[1], luid[2], luid[3]]);
                let high = u32::from_le_bytes([luid[4], luid[5], luid[6], luid[7]]);
                readings.push((format!("luid_0x{high:08X}_0x{low:08X}_phys_0"), value as f64));
            }
        }
        shutdown();
        readings
    }
}
