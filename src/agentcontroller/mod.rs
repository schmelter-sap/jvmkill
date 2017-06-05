pub trait AgentController : Action {
    fn setParameters(&mut self);
}

pub struct agentController {
    jvmti: ::env::JVMTIEnv,
    actions: Vec<Box<Action>>
}

impl agentController {
    pub fn new(ti: ::env::JVMTIEnv) -> Result<agentController, ::jvmti::jint> {
        self::heaphistogram::heapHistogram::new(ti).map(|action| {
            agentController {
                jvmti: ti,
                actions: vec![Box::new(action)],
            }
        })
    }
}

impl Action for agentController {
    fn onOOM(&self, jni_env: ::env::JNIEnv, resourceExhaustionFlags: ::jvmti::jint) {
        for action in &self.actions {
            action.onOOM(jni_env, resourceExhaustionFlags);
        }
    }
}

// TODO: use a raw monitor to provide the necessary synchronisation
unsafe impl Send for agentController {}
unsafe impl Sync for agentController {}

pub trait Action {
    // See https://docs.oracle.com/javase/8/docs/platform/jvmti/jvmti.html#jvmtiResourceExhaustionFlags
    fn onOOM(&self, jni_env: ::env::JNIEnv, resourceExhaustionFlags: ::jvmti::jint);
}

mod heaphistogram;
