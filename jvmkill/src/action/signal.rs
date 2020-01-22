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

use std::thread;
use std::time::Duration;

use libc::{c_int, getpid, kill};
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Signal {
    fn delay(&self) -> bool;

    fn kill(&self);

    fn signal(&self) -> c_int;
}

pub struct DefaultSignal {
    pub delay: bool,
    pub signal: c_int,
}

impl Signal for DefaultSignal {
    fn delay(&self) -> bool {
        return self.delay;
    }

    fn kill(&self) {
        unsafe { kill(getpid(), self.signal) };

        if self.delay {
            thread::sleep(Duration::from_millis(5000));
        }
    }

    fn signal(&self) -> i32 {
        return self.signal;
    }
}
