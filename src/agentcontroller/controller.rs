pub struct AgentController<'a> {
    jvmti: ::env::JvmTiEnv,
    heuristic: Box<super::Heuristic + 'a>,
    actions: Vec<Box<super::Action>>
}

impl<'a> AgentController<'a> {
    pub fn new(ti: ::env::JvmTiEnv, options: *mut ::std::os::raw::c_char) -> Result<Self, ::jvmti::jint> {
        super::heaphistogram::HeapHistogram::new(ti).map(|action| {
            let parms = super::parms::AgentParameters::parseParameters(options);
            Self {
                jvmti: ti,
                heuristic: Box::new(super::threshold::Threshold::new(parms.count_threshold, parms.time_threshold)),
                actions: vec![Box::new(action), Box::new(super::kill::Kill::new())],
            }
        })
    }

    #[cfg(test)]
    fn test_new(ti: ::env::JvmTiEnv, heuristic: Box<super::Heuristic + 'a>, actions: Vec<Box<super::Action>>) -> Self {
        Self {
            jvmti: ti,
            heuristic: heuristic,
            actions: actions,
        }
    }
}

impl<'a> super::MutAction for AgentController<'a> {
    fn on_oom(&mut self, jni_env: ::env::JniEnv, resourceExhaustionFlags: ::jvmti::jint) {
        if self.heuristic.on_oom() {
            for action in &self.actions {
                action.on_oom(jni_env, resourceExhaustionFlags);
            }
        } else {}
    }
}

unsafe impl<'a> Send for AgentController<'a> {}

unsafe impl<'a> Sync for AgentController<'a> {}

#[cfg(test)]
mod tests {
    use agentcontroller::MutAction;

    pub struct TestHeuristic {
        call_count: u32
    }

    impl TestHeuristic {
        pub fn new() -> Self {
            Self {
                call_count: 0,
            }
        }
    }

    impl super::super::Heuristic for TestHeuristic {
        fn on_oom(&mut self) -> bool {
            self.call_count += 1;
            self.call_count >= 2
        }
    }

    pub struct TestAction {}

    impl TestAction {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl super::super::Action for TestAction {
        fn on_oom(&self, _: ::env::JniEnv, _: ::jvmti::jint) {
            panic!("TestAction.on_oom")
        }
    }

    #[test]
    fn does_not_call_action_when_heuristic_returns_false() {
        let heuristic = Box::new(TestHeuristic::new());
        let mut ac = super::AgentController::test_new(::env::JvmTiEnv::new(dummy_jni()).unwrap(),
                                                      heuristic,
                                                      vec![Box::new(TestAction::new())]);
        ac.on_oom(dummy_jni_env(), 0);
    }

    #[test]
    #[should_panic(expected = "TestAction.on_oom")]
    fn calls_action_when_heuristic_returns_true() {
        let heuristic = Box::new(TestHeuristic::new());
        let mut ac = super::AgentController::test_new(::env::JvmTiEnv::new(dummy_jni()).unwrap(),
                                                      heuristic,
                                                      vec![Box::new(TestAction::new())]);
        ac.on_oom(dummy_jni_env(), 0);
        ac.on_oom(dummy_jni_env(), 0);
    }

    unsafe extern "C" fn test_get_env(vm: *mut ::jvmti::JavaVM,
                                      penv: *mut *mut ::std::os::raw::c_void,
                                      version: ::jvmti::jint)
                                      -> ::jvmti::jint {
        0
    }

    fn dummy_jni_env() -> ::env::JniEnv {
        ::env::JniEnv::new(::std::ptr::null_mut())
    }

    fn dummy_jni() -> *mut ::jvmti::JavaVM {
        &mut (&::jvmti::JNIInvokeInterface_ {
            reserved0: ::std::ptr::null_mut(),
            reserved1: ::std::ptr::null_mut(),
            reserved2: ::std::ptr::null_mut(),
            DestroyJavaVM: None,
            AttachCurrentThread: None,
            DetachCurrentThread: None,
            GetEnv: Some(test_get_env),
            AttachCurrentThreadAsDaemon: None,
        } as *const ::jvmti::JNIInvokeInterface_) as *mut *const ::jvmti::JNIInvokeInterface_
    }
}
