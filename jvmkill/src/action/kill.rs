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

use crate::action::Action;
use crate::action::signal::{DefaultSignal, Signal};
use crate::bindings::jint;

pub struct Kill<'k> {
    pub signal: &'k dyn Signal,
}

impl<'k> Kill<'k> {
    pub fn new() -> Self {
        return Self { signal: &DefaultSignal { signal: libc::SIGKILL, delay: false } };
    }
}

impl<'k> Action for Kill<'k> {
    fn execute(&self, _flags: jint) {
        eprintln!("\njvmkill is killing current process");
        self.signal.kill();
    }
}

#[cfg(test)]
mod tests {
    use mockall::Sequence;

    use crate::action::Action;
    use crate::action::kill::Kill;
    use crate::action::signal::MockSignal;

    #[test]
    fn execute() {
        let mut signal = MockSignal::new();
        let mut seq = Sequence::new();

        signal
            .expect_kill()
            .times(1)
            .in_sequence(&mut seq)
            .return_const(());

        let mut t = Kill::new();

        assert_eq!(t.signal.signal(), libc::SIGKILL);
        assert_eq!(t.signal.delay(), false);

        t.signal = &signal;
        t.execute(0);
    }
}


