use std::collections::VecDeque;

use crate::sampler::Snapshot;

const CAPACITY: usize = 60;

pub struct History {
    samples: VecDeque<[Option<f64>; 5]>,
}

impl History {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(CAPACITY),
        }
    }

    pub fn push(&mut self, snapshot: &Snapshot) {
        if self.samples.len() == CAPACITY {
            self.samples.pop_front();
        }
        self.samples.push_back([
            snapshot.cpu,
            snapshot.gpu,
            snapshot.ram,
            snapshot.download,
            snapshot.upload,
        ]);
    }

    pub fn points(&self, index: usize) -> impl Iterator<Item = Option<f64>> + '_ {
        self.samples.iter().map(move |sample| sample[index])
    }
}

#[cfg(test)]
mod tests {
    use super::History;
    use crate::sampler::Snapshot;

    #[test]
    fn retains_the_latest_sixty_samples_in_order() {
        let mut history = History::new();
        assert_eq!(history.points(0).count(), 0);
        for value in 0..62 {
            history.push(&Snapshot {
                cpu: Some(value as f64),
                ..Default::default()
            });
        }
        let points: Vec<_> = history.points(0).collect();
        assert_eq!(points.len(), 60);
        assert_eq!(points.first(), Some(&Some(2.0)));
        assert_eq!(points.last(), Some(&Some(61.0)));
    }

    #[test]
    fn keeps_metrics_separate_in_contract_order() {
        let mut history = History::new();
        history.push(&Snapshot {
            cpu: Some(1.0),
            gpu: Some(2.0),
            ram: Some(3.0),
            download: Some(4.0),
            upload: Some(5.0),
            ..Default::default()
        });
        for (index, expected) in [1.0, 2.0, 3.0, 4.0, 5.0].into_iter().enumerate() {
            assert_eq!(history.points(index).collect::<Vec<_>>(), vec![Some(expected)]);
        }
    }

    #[test]
    fn preserves_missing_values_and_zeroes() {
        let mut history = History::new();
        history.push(&Snapshot {
            cpu: Some(0.0),
            gpu: None,
            ..Default::default()
        });
        history.push(&Snapshot {
            cpu: None,
            gpu: Some(0.0),
            ..Default::default()
        });
        assert_eq!(history.points(0).collect::<Vec<_>>(), vec![Some(0.0), None]);
        assert_eq!(history.points(1).collect::<Vec<_>>(), vec![None, Some(0.0)]);
    }
}
