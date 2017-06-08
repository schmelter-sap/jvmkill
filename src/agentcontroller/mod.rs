pub struct AgentController<'a> {
    jvmti: ::env::JvmTIEnv,
    heuristic: Box<Heuristic + 'a>,
    actions: Vec<Box<Action>>
}

impl<'a> AgentController<'a> {
    pub fn new(ti: ::env::JvmTIEnv, options: *mut ::std::os::raw::c_char) -> Result<Self, ::jvmti::jint> {
        self::heaphistogram::HeapHistogram::new(ti).map(|action| Self {
            jvmti: ti,
            heuristic: Box::new(threshold::Threshold::new(parms::AgentParameters::parseParameters(options))),
            actions: vec![Box::new(action)],
        })
    }
}

impl<'a> Action for AgentController<'a> {
    fn on_oom(&self, jni_env: ::env::JniEnv, resourceExhaustionFlags: ::jvmti::jint) {
        for action in &self.actions {
            action.on_oom(jni_env, resourceExhaustionFlags);
        }
    }
}

unsafe impl<'a> Send for AgentController<'a> {}
unsafe impl<'a> Sync for AgentController<'a> {}

pub trait Action {
    // See https://docs.oracle.com/javase/8/docs/platform/jvmti/jvmti.html#jvmtiResourceExhaustionFlags
    fn on_oom(&self, jni_env: ::env::JniEnv, resourceExhaustionFlags: ::jvmti::jint);
}

pub trait Heuristic {
    fn on_oom(&mut self) -> bool;
}

mod heaphistogram;
mod threshold;
mod parms;
