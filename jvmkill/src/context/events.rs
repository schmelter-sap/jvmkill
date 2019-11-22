/*
 * Copyright 2015-2019 the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::time::Instant;

pub struct Events {
    events: circular_queue::CircularQueue<Instant>
}

impl Events {
    pub fn new(limit: usize) -> Events {
        return Events { events: circular_queue::CircularQueue::with_capacity(limit + 1) };
    }

    pub fn events_since(&mut self, since: Instant) -> usize {
        return self.events.iter()
            .filter(|&&i| i > since)
            .count();
    }

    pub fn record(&mut self) {
        self.events.push(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{Add, Sub};
    use std::time::{Duration, Instant};

    use crate::context::events::Events;

    #[test]
    fn filters_events() {
        let mut e = Events::new(1);

        e.record();

        assert_eq!(e.events_since(Instant::now().add(Duration::from_secs(10))), 0);
        assert_eq!(e.events_since(Instant::now().sub(Duration::from_secs(10))), 1);
    }

    #[test]
    fn records_beyond_limit() {
        let mut e = Events::new(2);

        for _ in 1..5 {
            e.record();
        }

        assert_eq!(e.events_since(Instant::now().sub(Duration::from_secs(10))), 3)
    }

    #[test]
    fn records_events() {
        let mut e = Events::new(1);

        e.record();

        assert_eq!(e.events_since(Instant::now().sub(Duration::from_secs(10))), 1)
    }
}
