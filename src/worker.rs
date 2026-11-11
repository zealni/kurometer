use crate::{
    sampler::{Sampler, Snapshot},
    settings::Settings,
};
use std::{
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread,
    time::{Duration, Instant},
};
pub struct Worker {
    commands: Sender<Option<Settings>>,
    readings: Receiver<(Snapshot, Instant)>,
}
/// Adapter LUIDs change across boots, so settings store the adapter name.
fn resolve(name: &str) -> String {
    crate::sampler::gpus()
        .into_iter()
        .find(|(_, label)| label.eq_ignore_ascii_case(name))
        .map(|(id, _)| id)
        .unwrap_or_default()
}
impl Worker {
    pub fn new(config: Settings) -> Self {
        let (tx, rx) = mpsc::channel::<Option<Settings>>();
        let (out, readings) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let mut config = config;
            let mut sampler = Sampler::new(config.adapter, resolve(&config.gpu));
            loop {
                let mut value = sampler.sample();
                if config.gpu_temperature {
                    value.gpu_temperatures = crate::temperature::gpu_temperatures();
                }
                let _ = out.try_send((value, Instant::now()));
                match rx.recv_timeout(Duration::from_secs(config.interval as u64)) {
                    Ok(Some(next)) => {
                        config = next;
                        sampler = Sampler::new(config.adapter, resolve(&config.gpu));
                    }
                    Ok(None) | Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => {}
                }
            }
        });
        Self {
            commands: tx,
            readings,
        }
    }
    pub fn configure(&self, config: Settings) {
        let _ = self.commands.send(Some(config));
    }
    pub fn latest(&self) -> Option<(Snapshot, Instant)> {
        let mut latest = None;
        while let Ok(value) = self.readings.try_recv() {
            latest = Some(value);
        }
        latest
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.commands.send(None);
    }
}
