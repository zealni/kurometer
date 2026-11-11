use crate::metrics::{busiest_engine, cpu_busy, transfer_rate};
use std::{collections::BTreeMap, mem::size_of, ptr::null_mut, time::Instant};
use windows::{
    Win32::{
        Foundation::FILETIME,
        NetworkManagement::{IpHelper::*, Ndis::IfOperStatusUp},
        System::{Performance::*, SystemInformation::*, Threading::GetSystemTimes},
    },
    core::{PCWSTR, w},
};
#[derive(Clone, Debug)]
pub struct Adapter {
    pub id: u64,
    pub name: String,
    pub active: bool,
    pub physical: bool,
}
pub fn gpus() -> Vec<(String, String)> {
    use windows::Win32::Graphics::Dxgi::*;
    unsafe {
        let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() else {
            return vec![];
        };
        let mut result = Vec::new();
        let mut index = 0;
        while let Ok(adapter) = factory.EnumAdapters1(index) {
            if let Ok(desc) = adapter.GetDesc1() {
                if desc.Flags & 2 == 0 {
                    let name = String::from_utf16_lossy(
                        &desc.Description
                            [..desc.Description.iter().position(|c| *c == 0).unwrap_or(128)],
                    );
                    result.push((
                        format!(
                            "luid_0x{:08X}_0x{:08X}_phys_0",
                            desc.AdapterLuid.HighPart as u32, desc.AdapterLuid.LowPart
                        ),
                        name,
                    ));
                }
            }
            index += 1;
        }
        result
    }
}
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub cpu: Option<f64>,
    pub cpu_mhz: Option<f64>,
    pub disk_read: Option<f64>,
    pub disk_write: Option<f64>,
    pub ram: Option<f64>,
    pub used: u64,
    pub total: u64,
    pub download: Option<f64>,
    pub upload: Option<f64>,
    pub gpu: Option<f64>,
    pub gpu_id: String,
    pub gpus: Vec<(String, Option<f64>)>,
    pub gpu_temperatures: Vec<(String, f64)>,
    pub network_status: String,
}
pub fn adapters() -> Vec<Adapter> {
    unsafe {
        let mut table = null_mut();
        if GetIfTable2(&mut table).is_err() {
            return vec![];
        }
        let rows =
            std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize);
        let result = rows
            .iter()
            .filter(|r| r.Type != 24 && r.InterfaceAndOperStatusFlags._bitfield & 8 == 0)
            .map(|r| Adapter {
                id: r.InterfaceLuid.Value,
                name: String::from_utf16_lossy(
                    &r.Alias[..r
                        .Alias
                        .iter()
                        .position(|c| *c == 0)
                        .unwrap_or(r.Alias.len())],
                ),
                active: r.OperStatus == IfOperStatusUp,
                physical: (r.InterfaceAndOperStatusFlags._bitfield & 1) != 0,
            })
            .collect();
        FreeMibTable(table.cast());
        result
    }
}
fn times() -> Option<[u64; 3]> {
    unsafe {
        let (mut idle, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)).ok()?;
        let cv = |t: FILETIME| ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64;
        Some([cv(idle), cv(kernel), cv(user)])
    }
}
fn disk_rate(previous: &mut Option<(u64, Instant)>, current: Option<u64>, now: Instant) -> Option<f64> {
    let rate = previous.zip(current).and_then(|((old, time), new)| {
        transfer_rate(old, new, now.duration_since(time).as_secs_f64())
    });
    *previous = current.map(|value| (value, now));
    rate
}

unsafe fn raw_counter(counter: PDH_HCOUNTER) -> Option<u64> {
    let mut value = PDH_RAW_COUNTER::default();
    if unsafe { PdhGetRawCounterValue(counter, None, &mut value) } != 0 || value.CStatus > 1 {
        return None;
    }
    u64::try_from(value.FirstValue).ok()
}


pub struct Sampler {
    previous_cpu: Option<[u64; 3]>,
    previous_net: Option<(u64, u64, Instant)>,
    previous_disk_read: Option<(u64, Instant)>,
    previous_disk_write: Option<(u64, Instant)>,
    pub adapter: u64,
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
    cpu_frequency: PDH_HCOUNTER,
    disk_read: PDH_HCOUNTER,
    disk_write: PDH_HCOUNTER,
    buffer: Vec<usize>,
    pub gpu_id: String,
}
impl Sampler {
    pub fn new(adapter: u64, gpu_id: String) -> Self {
        unsafe {
            let mut query = Default::default();
            let mut counter = Default::default();
            let mut cpu_frequency = Default::default();
            let mut disk_read = Default::default();
            let mut disk_write = Default::default();
            if PdhOpenQueryW(PCWSTR::null(), 0, &mut query) == 0 {
                PdhAddEnglishCounterW(query, w!("\\Processor Information(_Total)\\Processor Frequency"), 0, &mut cpu_frequency);
                PdhAddEnglishCounterW(query, w!("\\PhysicalDisk(_Total)\\Disk Read Bytes/sec"), 0, &mut disk_read);
                PdhAddEnglishCounterW(query, w!("\\PhysicalDisk(_Total)\\Disk Write Bytes/sec"), 0, &mut disk_write);
                PdhAddEnglishCounterW(
                    query,
                    w!("\\GPU Engine(*)\\Utilization Percentage"),
                    0,
                    &mut counter,
                );
                PdhCollectQueryData(query);
            }
            Self {
                previous_cpu: None,
                previous_net: None,
                previous_disk_read: None,
                previous_disk_write: None,
                adapter,
                query,
                counter,
                cpu_frequency,
                disk_read,
                disk_write,
                buffer: vec![],
                gpu_id,
            }
        }
    }
    pub fn sample(&mut self) -> Snapshot {
        unsafe {
            let mut s = Snapshot::default();
            let now = Instant::now();
            let current = times();
            s.cpu = self.previous_cpu.zip(current).and_then(|(p, c)| cpu_busy(p, c));
            self.previous_cpu = current;
            let mut memory = MEMORYSTATUSEX {
                dwLength: size_of::<MEMORYSTATUSEX>() as u32,
                ..Default::default()
            };
            if GlobalMemoryStatusEx(&mut memory).is_ok() && memory.ullTotalPhys > 0 {
                s.total = memory.ullTotalPhys;
                s.used = memory.ullTotalPhys - memory.ullAvailPhys;
                s.ram = Some(s.used as f64 * 100.0 / s.total as f64);
            }
            let mut row = MIB_IF_ROW2::default();
            row.InterfaceLuid.Value = self.adapter;
            if self.adapter != 0
                && GetIfEntry2(&mut row).is_ok()
                && row.OperStatus == IfOperStatusUp
            {
                if let Some((rx, tx, time)) = self.previous_net {
                    let seconds = now.duration_since(time).as_secs_f64();
                    if seconds <= 15.0 {
                        s.download = transfer_rate(rx, row.InOctets, seconds);
                        s.upload = transfer_rate(tx, row.OutOctets, seconds);
                    }
                }
                self.previous_net = Some((row.InOctets, row.OutOctets, now));
                s.network_status = if s.download.is_some() {
                    "Connected"
                } else {
                    "Collecting"
                }
                .into();
            } else {
                self.previous_net = None;
                s.network_status = "Disconnected / select adapter".into();
            }
            let mut groups = BTreeMap::<String, Vec<(String, f64)>>::new();
            if PdhCollectQueryData(self.query) == 0 {
                let mut frequency = PDH_FMT_COUNTERVALUE::default();
                if PdhGetFormattedCounterValue(self.cpu_frequency, PDH_FMT_DOUBLE, None, &mut frequency) == 0
                    && frequency.CStatus <= 1
                {
                    let mhz = frequency.Anonymous.doubleValue;
                    if mhz.is_finite() && mhz > 0.0 {
                        s.cpu_mhz = Some(mhz);
                    }
                }
                s.disk_read = disk_rate(&mut self.previous_disk_read, raw_counter(self.disk_read), now);
                s.disk_write = disk_rate(&mut self.previous_disk_write, raw_counter(self.disk_write), now);
                let (mut bytes, mut count) = (0, 0);
                let status = PdhGetFormattedCounterArrayW(
                    self.counter,
                    PDH_FMT_DOUBLE,
                    &mut bytes,
                    &mut count,
                    None,
                );
                if status == 0x800007d2 && bytes > 0 {
                    self.buffer.resize(
                        (bytes as usize + size_of::<usize>() - 1) / size_of::<usize>(),
                        0,
                    );
                    if PdhGetFormattedCounterArrayW(
                        self.counter,
                        PDH_FMT_DOUBLE,
                        &mut bytes,
                        &mut count,
                        Some(self.buffer.as_mut_ptr().cast()),
                    ) == 0
                    {
                        for item in std::slice::from_raw_parts(
                            self.buffer.as_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>(),
                            count as usize,
                        ) {
                            let name = item.szName.to_string().unwrap_or_default();
                            if let Some(start) = name.find("luid_") {
                                let engine = &name[start..];
                                if let Some(end) = engine.find("_eng_") {
                                    let rows = groups.entry(engine[..end].into()).or_default();
                                    if item.FmtValue.CStatus <= 1 {
                                        rows.push((
                                            engine.into(),
                                            item.FmtValue.Anonymous.doubleValue,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                self.previous_disk_read = None;
                self.previous_disk_write = None;
            }
            if self.gpu_id.is_empty() {
                if let Some((id, _)) = groups.iter().find(|(_, rows)| !rows.is_empty()) {
                    self.gpu_id = id.clone();
                }
            }
            s.gpu = groups
                .get(&self.gpu_id)
                .and_then(|rows| busiest_engine(rows));
            s.gpu_id = self.gpu_id.clone();
            s.gpus = groups
                .into_iter()
                .map(|(id, rows)| (id, busiest_engine(&rows)))
                .collect();
            s
        }
    }
}
impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            PdhCloseQuery(self.query);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn disk_rate_handles_initial_sample_reset_and_unavailable() {
        let now = Instant::now();
        let mut previous = None;
        assert_eq!(disk_rate(&mut previous, Some(100), now), None);
        assert_eq!(disk_rate(&mut previous, Some(1100), now + Duration::from_secs(2)), Some(500.0));
        assert_eq!(disk_rate(&mut previous, Some(10), now + Duration::from_secs(3)), None);
        assert_eq!(disk_rate(&mut previous, None, now + Duration::from_secs(4)), None);
        assert_eq!(disk_rate(&mut previous, Some(20), now + Duration::from_secs(5)), None);
    }
}
