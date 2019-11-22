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

use std::ops::Sub;
use std::os::raw::c_char;
use std::time::{Duration, Instant};

use crate::context::events::Events;
use crate::context::Parameters;

pub struct Context {
    events: Events,
    pub parameters: Parameters,
}

impl Context {
    pub fn new(options: *const c_char) -> Context {
        let p = Parameters::new(options);
        let e = Events::new(p.count_threshold);

        return Context { events: e, parameters: p };
    }

    pub fn record(&mut self) -> bool {
        self.events.record();
        let count = self.events.events_since(Instant::now().sub(Duration::from_secs(self.parameters.time_threshold as u64)));
        eprintln!("Resource Exhausted! ({}/{})", count, self.parameters.count_threshold);
        return count > self.parameters.count_threshold;
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use crate::context::Context;

    #[test]
    fn does_not_trigger() {
        assert_eq!(create("count=100,time=100").record(), false);
    }

    #[test]
    fn triggers() {
        assert_eq!(create("count=0").record(), true);
    }

    fn create(s: &str) -> Context {
        let options = CString::new(s)
            .expect("cannot convert to CString");

        return Context::new(options.as_ptr());
    }
}
