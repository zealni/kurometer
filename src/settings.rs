#[derive(Clone, Debug)]
pub struct Settings {
    pub adapter: u64,
    pub gpu: String,
    pub interval: u32,
    pub bits: bool,
    pub theme: u32,
    pub visible: [bool; 5],
    pub order: String,
    pub startup: bool,
    pub display: String,
    pub width: i32,
    pub position: i32,
    pub offset: i32,
    pub locked: bool,
    pub font: String,
    pub transparent: bool,
    pub palette: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            adapter: 0,
            gpu: String::new(),
            interval: 1,
            bits: false,
            theme: 0,
            visible: [true; 5],
            order: "CPU,GPU,RAM,Download,Upload".into(),
            startup: false,
            display: String::new(),
            width: 500,
            position: -1,
            offset: -1,
            locked: false,
            font: "Segoe UI".into(),
            transparent: false,
            palette: 0,
        }
    }
}
impl Settings {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut s = Self::default();
        for line in text.lines() {
            let (key, value) = line.split_once('=').ok_or("Malformed settings line")?;
            match key {
                "adapter" => s.adapter = value.parse().map_err(|_| "Invalid adapter")?,
                "gpu" => {
                    if value.len() > 200 || value.contains(['\r', '\n']) {
                        return Err("Invalid GPU identifier".into());
                    }
                    s.gpu = value.into();
                }
                "interval" => {
                    s.interval = value.parse().map_err(|_| "Invalid interval")?;
                    if ![1, 2, 5].contains(&s.interval) {
                        return Err("Unsupported interval".into());
                    }
                }
                "bits" => s.bits = value.parse().map_err(|_| "Invalid unit")?,
                "theme" => {
                    s.theme = value.parse().map_err(|_| "Invalid theme")?;
                    if s.theme > 2 {
                        return Err("Invalid theme".into());
                    }
                }
                "visible" => {
                    if value.len() != 5
                        || !value.bytes().all(|c| c == b'0' || c == b'1')
                        || !value.contains('1')
                    {
                        return Err("Select at least one metric".into());
                    }
                    for (i, c) in value.bytes().enumerate() {
                        s.visible[i] = c == b'1';
                    }
                }
                "startup" => s.startup = value.parse().map_err(|_| "Invalid startup preference")?,
                "order" => {
                    let mut values = value.split(',').collect::<Vec<_>>();
                    values.sort_unstable();
                    if values != ["CPU", "Download", "GPU", "RAM", "Upload"] {
                        return Err("Order must contain each metric exactly once".into());
                    }
                    s.order = value.into();
                }
                "display" => {
                    if value.len() > 100 || value.contains(['\r', '\n']) {
                        return Err("Invalid display".into());
                    }
                    s.display = value.into();
                }
                "width" => {
                    s.width = value.parse().map_err(|_| "Invalid width")?;
                    if !(260..=1200).contains(&s.width) {
                        return Err("Width must be between 260 and 1200 pixels".into());
                    }
                }
                "position" => {
                    s.position = value.parse().map_err(|_| "Invalid position")?;
                    if s.position < -1 || s.position > 100000 {
                        return Err("Invalid position".into());
                    }
                }
                "locked" => s.locked = value.parse().map_err(|_| "Invalid lock preference")?,
                "offset" => {
                    s.offset = value.parse().map_err(|_| "Invalid offset")?;
                    if s.offset < -1 || s.offset > 100000 {
                        return Err("Invalid offset".into());
                    }
                }
                "font" => {
                    if !["Segoe UI", "Arial", "Consolas"].contains(&value) {
                        return Err("Unsupported meter font".into());
                    }
                    s.font = value.into();
                }
                "transparent" => {
                    s.transparent = value.parse().map_err(|_| "Invalid transparency")?
                }
                "palette" => {
                    s.palette = value.parse().map_err(|_| "Invalid palette")?;
                    if s.palette > 2 {
                        return Err("Invalid palette".into());
                    }
                }
                _ => return Err(format!("Unknown setting: {key}")),
            }
        }
        Ok(s)
    }
    pub fn serialize(&self) -> String {
        format!(
            "adapter={}\ngpu={}\ninterval={}\nbits={}\ntheme={}\nvisible={}\norder={}\nstartup={}\ndisplay={}\nwidth={}\nposition={}\nlocked={}\noffset={}\nfont={}\ntransparent={}\npalette={}\n",
            self.adapter,
            self.gpu,
            self.interval,
            self.bits,
            self.theme,
            self.visible
                .iter()
                .map(|v| if *v { '1' } else { '0' })
                .collect::<String>(),
            self.order,
            self.startup,
            self.display,
            self.width,
            self.position,
            self.locked,
            self.offset,
            self.font,
            self.transparent,
            self.palette,
        )
    }
}
pub fn path() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default())
        .join("TaskbarSystemMonitor")
        .join("settings.ini")
}
pub fn load() -> Result<Settings, String> {
    match std::fs::read_to_string(path()) {
        Ok(text) => Settings::parse(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e.to_string()),
    }
}
pub fn save(settings: &Settings) -> Result<(), String> {
    let path = path();
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, settings.serialize()).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn MoveFileExW(a: *const u16, b: *const u16, flags: u32) -> i32;
        }
        let a: Vec<u16> = temporary.as_os_str().encode_wide().chain(Some(0)).collect();
        let b: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        if unsafe { MoveFileExW(a.as_ptr(), b.as_ptr(), 9) } == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preferences_round_trip() {
        let original = Settings {
            adapter: 42,
            gpu: "luid_1_phys_0".into(),
            interval: 5,
            bits: true,
            theme: 2,
            visible: [true, false, true, true, true],
            order: "CPU,GPU,RAM,Download,Upload".into(),
            startup: false,
            display: String::new(),
            width: 480,
            position: 3,
            locked: true,
            offset: 120,
            font: "Consolas".into(),
            transparent: true,
            palette: 2,
        };
        let parsed = Settings::parse(&original.serialize()).unwrap();
        assert_eq!(parsed.adapter, 42);
        assert_eq!(parsed.interval, 5);
        assert!(parsed.bits);
        assert!(!parsed.visible[1]);
        assert_eq!(parsed.width, 480);
        assert_eq!(parsed.position, 3);
        assert_eq!(parsed.offset, 120);
        assert!(parsed.locked);
        assert_eq!(parsed.font, "Consolas");
        assert!(parsed.transparent);
        assert_eq!(parsed.palette, 2);
    }
    #[test]
    fn gpu_adapter_name_survives_round_trip() {
        // LUIDs change across boots, so the saved identity is the display name.
        let mut original = Settings::default();
        original.gpu = "Intel(R) Iris(R) Xe Graphics".into();
        let parsed = Settings::parse(&original.serialize()).unwrap();
        assert_eq!(parsed.gpu, "Intel(R) Iris(R) Xe Graphics");
    }
    #[test]
    fn invalid_intervals_and_empty_meter_rejected() {
        assert!(Settings::parse("interval=0\n").is_err());
        assert!(Settings::parse("visible=00000\n").is_err());
        assert!(Settings::parse("order=CPU,CPU,RAM,Download,Upload\n").is_err());
        assert!(Settings::parse("order=Upload,Download,RAM,GPU,CPU\n").is_ok());
    }
}
