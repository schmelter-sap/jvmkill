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

    pub fn setSignal(&mut self, signal: c_int) {
        self.signal = signal;
    }
}

impl super::Action for Kill {
    fn on_oom(&self, _: ::env::JniEnv, _: ::jvmti::jint) {
        eprintln!("jvmkill killing current process");
        unsafe {
            kill(getpid(), self.signal);
        }
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
        kill.on_oom(::env::JniEnv::new(::std::ptr::null_mut()), 0);

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
            signal::sigaction(signal::SIGUSR1, &sig_action);
        }
    }
}
