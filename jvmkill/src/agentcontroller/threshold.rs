/*
 * Copyright (c) 2015-2022 the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use chrono::prelude::*;

pub struct Threshold {
    time_threshold: i64,
    count_threshold: usize,
    // circular buffer containing the timestamps of up to count_threshold + 1 OOMs
    events: Vec<i64>,
    event_index: usize,
}

impl Threshold {
    pub fn new(count_threshold: usize, time_threshold: i64) -> Threshold {
        let mut t = Threshold {
            time_threshold: 1000 * (time_threshold as i64),
            count_threshold,
            events: Vec::with_capacity(count_threshold + 1),
            event_index: 0,
        };

        // prefill with a safe value
        for _ in 0..count_threshold + 1 {
            t.events.push(0);
        }

        t
    }

    fn add_event(&mut self) {
        self.events[self.event_index] = millis();
        self.event_index += 1;
        if self.event_index > self.count_threshold {
            self.event_index = 0;
        }
    }

    fn count_events(&self) -> usize {
        let horizon = millis() - self.time_threshold;

        self.events.iter().filter(|&&t| t >= horizon).count()
    }
}

fn millis() -> i64 {
    Utc::now().timestamp_millis()
}

impl super::Heuristic for Threshold {
    fn on_oom(&mut self) -> bool {
        self.add_event();
        let eventCount = self.count_events();
        eprintln!(
            "ResourceExhausted! ({}/{})",
            eventCount, self.count_threshold
        );
        eventCount > self.count_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::super::Heuristic;
    use std::{thread, time};

    #[test]
    fn triggers_if_threshold_exceeded() {
        let mut threshold = super::Threshold::new(2, 3);

        assert!(!threshold.on_oom());
        assert!(!threshold.on_oom());
        assert!(threshold.on_oom());
    }

    #[test]
    fn does_not_trigger_if_threshold_not_exceeded() {
        let mut threshold = super::Threshold::new(2, 1);

        assert!(!threshold.on_oom());
        assert!(!threshold.on_oom());

        thread::sleep(time::Duration::from_millis(1100));

        assert!(!threshold.on_oom());
        assert!(!threshold.on_oom());
    }
}
