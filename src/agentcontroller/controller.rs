pub struct AgentController<'a> {
    jvmti: ::env::JvmTIEnv,
    heuristic: Box<super::Heuristic + 'a>,
    actions: Vec<Box<super::Action>>
}

impl<'a> AgentController<'a> {
    pub fn new(ti: ::env::JvmTIEnv, options: *mut ::std::os::raw::c_char) -> Result<Self, ::jvmti::jint> {
        super::heaphistogram::HeapHistogram::new(ti).map(|action| Self {
            jvmti: ti,
            heuristic: Box::new(super::threshold::Threshold::new(super::parms::AgentParameters::parseParameters(options))),
            actions: vec![Box::new(action)],
        })
    }
}

impl<'a> super::Action for AgentController<'a> {
    fn on_oom(&self, jni_env: ::env::JniEnv, resourceExhaustionFlags: ::jvmti::jint) {
        for action in &self.actions {
            action.on_oom(jni_env, resourceExhaustionFlags);
        }
    }
}

unsafe impl<'a> Send for AgentController<'a> {}
unsafe impl<'a> Sync for AgentController<'a> {}
