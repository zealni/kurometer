#[derive(Clone, Debug)]
pub struct Settings {
    pub adapter: u64,
    pub gpu: String,
    pub all_gpus: bool,
    pub gpu_temperature: bool,
    pub cpu_speed: bool,
    pub disk_activity: bool,
    pub interval: u32,
    pub bits: bool,
    pub theme: u32,
    pub visible: [bool; 5],
    pub labels: [bool; 5],
    pub charts: [bool; 5],
    pub ram_format: u32,
    pub order: String,
    pub startup: bool,
    pub display: String,
    pub width: i32,
    pub position: i32,
    pub offset: i32,
    pub locked: bool,
    pub font: String,
    pub font_size: u32,
    pub transparent: bool,
    pub palette: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            adapter: 0,
            gpu: String::new(),
            all_gpus: false,
            gpu_temperature: false,
            cpu_speed: false,
            disk_activity: false,
            interval: 1,
            bits: false,
            theme: 0,
            visible: [true; 5],
            labels: [true; 5],
            charts: [false; 5],
            ram_format: 0,
            order: "CPU,GPU,RAM,Download,Upload".into(),
            startup: false,
            display: String::new(),
            width: 500,
            position: -1,
            offset: -1,
            locked: false,
            font: "Segoe UI Variable".into(),
            font_size: 0,
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
                "all_gpus" => s.all_gpus = value.parse().map_err(|_| "Invalid GPU selection")?,
                "gpu_temperature" => s.gpu_temperature = value.parse().map_err(|_| "Invalid GPU temperature preference")?,
                "cpu_speed" => s.cpu_speed = value.parse().map_err(|_| "Invalid CPU speed preference")?,
                "disk_activity" => s.disk_activity = value.parse().map_err(|_| "Invalid disk activity preference")?,
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
                "labels" => {
                    if value.len() != 5 || !value.bytes().all(|c| c == b'0' || c == b'1') {
                        return Err("Invalid meter labels".into());
                    }
                    for (i, c) in value.bytes().enumerate() {
                        s.labels[i] = c == b'1';
                    }
                }
                "charts" => {
                    if value.len() != 5 || !value.bytes().all(|c| c == b'0' || c == b'1') {
                        return Err("Invalid chart selection".into());
                    }
                    for (i, c) in value.bytes().enumerate() {
                        s.charts[i] = c == b'1';
                    }
                }
                "ram_format" => {
                    s.ram_format = value.parse().map_err(|_| "Invalid RAM format")?;
                    if s.ram_format > 2 {
                        return Err("Invalid RAM format".into());
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
                    if value.is_empty() || value.encode_utf16().count() > 31 || value.chars().any(char::is_control) {
                        return Err("Unsupported meter font".into());
                    }
                    s.font = value.into();
                }
                "font_size" => {
                    s.font_size = value.parse().map_err(|_| "Invalid font size")?;
                    if s.font_size != 0 && !(8..=32).contains(&s.font_size) {
                        return Err("Font size must be Auto or 8–32 pixels".into());
                    }
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
            "adapter={}\ngpu={}\nall_gpus={}\ngpu_temperature={}\ncpu_speed={}\ndisk_activity={}\ninterval={}\nbits={}\ntheme={}\nvisible={}\nlabels={}\ncharts={}\nram_format={}\norder={}\nstartup={}\ndisplay={}\nwidth={}\nposition={}\nlocked={}\noffset={}\nfont={}\nfont_size={}\ntransparent={}\npalette={}\n",
            self.adapter,
            self.gpu,
            self.all_gpus,
            self.gpu_temperature,
            self.cpu_speed,
            self.disk_activity,
            self.interval,
            self.bits,
            self.theme,
            self.visible
                .iter()
                .map(|v| if *v { '1' } else { '0' })
                .collect::<String>(),
            self.labels.iter().map(|v| if *v { '1' } else { '0' }).collect::<String>(),
            self.charts.iter().map(|v| if *v { '1' } else { '0' }).collect::<String>(),
            self.ram_format,
            self.order,
            self.startup,
            self.display,
            self.width,
            self.position,
            self.locked,
            self.offset,
            self.font,
            self.font_size,
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
    fn installed_font_names_are_preserved() {
        let settings = Settings::parse("font=Arial Narrow\n").unwrap();
        assert_eq!(Settings::parse(&settings.serialize()).unwrap().font, "Arial Narrow");
        assert!(Settings::parse("font=\n").is_err());
    }
    #[test]
    fn font_size_round_trips_and_rejects_out_of_range() {
        let settings = Settings::parse("font_size=14\n").unwrap();
        assert_eq!(Settings::parse(&settings.serialize()).unwrap().font_size, 14);
        assert!(Settings::parse("font_size=7\n").is_err());
        assert!(Settings::parse("font_size=33\n").is_err());
    }
    #[test]
    fn extra_meter_preferences_round_trip() {
        let parsed = Settings::parse("cpu_speed=true\ndisk_activity=true\n").unwrap();
        let restored = Settings::parse(&parsed.serialize()).unwrap();
        assert!(restored.cpu_speed && restored.disk_activity);
        assert!(Settings::parse("cpu_speed=maybe\n").is_err());
    }
    #[test]
    fn chart_selection_survives_restart() {
        let settings = Settings::parse("charts=10101\n").unwrap();
        assert_eq!(settings.charts, [true, false, true, false, true]);
        assert_eq!(Settings::parse(&settings.serialize()).unwrap().charts, settings.charts);
        assert!(Settings::parse("charts=10102\n").is_err());
    }
    #[test]
    fn meter_formats_round_trip_and_reject_invalid_values() {
        let parsed = Settings::parse("labels=10101\nram_format=2\n").unwrap();
        assert_eq!(parsed.labels, [true, false, true, false, true]);
        assert_eq!(Settings::parse(&parsed.serialize()).unwrap().ram_format, 2);
        assert!(Settings::parse("labels=1010\n").is_err());
        assert!(Settings::parse("ram_format=3\n").is_err());
    }
    #[test]
    fn preferences_round_trip() {
        let original = Settings {
            adapter: 42,
            gpu: "luid_1_phys_0".into(),
            all_gpus: false,
            gpu_temperature: false,
            cpu_speed: false,
            disk_activity: false,
            interval: 5,
            bits: true,
            theme: 2,
            visible: [true, false, true, true, true],
            labels: [true; 5],
            charts: [false; 5],
            ram_format: 0,
            order: "CPU,GPU,RAM,Download,Upload".into(),
            startup: false,
            display: String::new(),
            width: 480,
            position: 3,
            locked: true,
            offset: 120,
            font: "Consolas".into(),
            font_size: 0,
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
