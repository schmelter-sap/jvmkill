/*
 * Copyright (c) 2017 the original author or authors.
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

use libc::c_int;
use libc::SIGKILL;
use libc::getpid;
use libc::kill;

pub struct Kill {
    signal: c_int
}

impl Kill {
    pub fn new() -> Self {
        Self {
            signal: SIGKILL,
        }
    }

    #[cfg(test)]
    pub fn setSignal(&mut self, signal: c_int) {
        self.signal = signal;
    }
}

impl ::std::fmt::Display for Kill {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "Kill")
    }
}

impl super::Action for Kill {
    fn on_oom(&self, _: ::env::JniEnv, _: ::jvmti::jint) -> Result<(), ::err::Error> {
        eprintln!("\njvmkill killing current process");
        unsafe {
            kill(getpid(), self.signal);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate nix;

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::{time, thread};
    use super::super::Action;
    use libc::SIGUSR1;
    use self::nix::sys::signal;

    lazy_static! {
        static ref FIRED: AtomicBool = AtomicBool::new(false);
    }

    extern fn handler(sig: i32) {
        if sig == SIGUSR1 {
            FIRED.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn sends_signal() {
        set_signal_handler();

        let mut kill = super::Kill::new();
        kill.setSignal(SIGUSR1);
        kill.on_oom(::env::JniEnv::new(::std::ptr::null_mut()), 0).expect("on_oom failed");

        // Allow time for signal to be dispatched.
        thread::sleep(time::Duration::from_millis(100));

        assert!(FIRED.load(Ordering::SeqCst));

        reset_signal_handler();
    }

    fn set_signal_handler() {
        sig_action(signal::SigHandler::Handler(handler));
    }

    fn reset_signal_handler() {
        sig_action(signal::SigHandler::SigDfl);
    }

    fn sig_action(action: signal::SigHandler) {
        let sig_action = signal::SigAction::new(action,
                                                signal::SaFlags::empty(),
                                                signal::SigSet::empty());
        unsafe {
            signal::sigaction(signal::SIGUSR1, &sig_action).expect("sigaction failed");
        }
    }
}
