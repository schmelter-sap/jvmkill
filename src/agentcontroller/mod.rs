pub mod controller;

pub trait MutAction {
    // See https://docs.oracle.com/javase/8/docs/platform/jvmti/jvmti.html#jvmtiResourceExhaustionFlags
    fn on_oom(&mut self, jni_env: ::env::JniEnv, resourceExhaustionFlags: ::jvmti::jint);
}

mod heaphistogram;
mod kill;
mod threshold;
mod parms;

trait Heuristic {
    fn on_oom(&mut self) -> bool;
}

trait Action {
    // See https://docs.oracle.com/javase/8/docs/platform/jvmti/jvmti.html#jvmtiResourceExhaustionFlags
    fn on_oom(&self, jni_env: ::env::JniEnv, resourceExhaustionFlags: ::jvmti::jint);
}
