pub mod controller;
mod heaphistogram;
mod threshold;
mod parms;

pub trait Action {
    // See https://docs.oracle.com/javase/8/docs/platform/jvmti/jvmti.html#jvmtiResourceExhaustionFlags
    fn on_oom(&self, jni_env: ::env::JniEnv, resourceExhaustionFlags: ::jvmti::jint);
}

pub trait Heuristic {
    fn on_oom(&mut self) -> bool;
}
