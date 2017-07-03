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

pub struct AgentController<'a> {
    #[allow(dead_code)] // TODO: revisit this once port is complete
    jvmti: ::env::JvmTiEnv,
    heuristic: Box<super::Heuristic + 'a>,
    actions: Vec<Box<super::Action>>
}

impl<'a> AgentController<'a> {
    pub fn new(ti: ::env::JvmTiEnv, options: *mut ::std::os::raw::c_char) -> Result<Self, ::jvmti::jint> {
        let parms = super::parms::AgentParameters::parseParameters(options);

        let mut ac = Self {
            jvmti: ti,
            heuristic: Box::new(super::threshold::Threshold::new(parms.count_threshold, parms.time_threshold)),
            actions: Vec::new(),
        };

        if parms.print_heap_histogram {
            ac.actions.push(Box::new(super::heaphistogram::HeapHistogram::new(ti, parms.heap_histogram_max_entries)?));
        }

        ac.actions.push(Box::new(super::kill::Kill::new()));

        Ok(ac)
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
    fn on_oom(&mut self, jni_env: ::env::JniEnv, resource_exhaustion_flags: ::jvmti::jint) {
        const heap_exhausted: ::jvmti::jint = ::jvmti::JVMTI_RESOURCE_EXHAUSTED_JAVA_HEAP as ::jvmti::jint;
        const threads_exhausted: ::jvmti::jint = ::jvmti::JVMTI_RESOURCE_EXHAUSTED_THREADS as ::jvmti::jint;
        const oom_error: ::jvmti::jint = ::jvmti::JVMTI_RESOURCE_EXHAUSTED_OOM_ERROR as ::jvmti::jint;

        if resource_exhaustion_flags & heap_exhausted == heap_exhausted {
            eprintln!("\nResource exhaustion event: the JVM was unable to allocate memory from the heap.");
        }
        if resource_exhaustion_flags & threads_exhausted == threads_exhausted {
            eprintln!("\nResource exhaustion event: the JVM was unable to create a thread.");
        }

        if self.heuristic.on_oom() {
            for action in &self.actions {
                action.on_oom(jni_env, resource_exhaustion_flags);
            }
        } else if resource_exhaustion_flags & oom_error == oom_error {
            eprintln!("\nThe JVM is about to throw a java.lang.OutOfMemoryError.");
        }
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

    unsafe extern "C" fn test_get_env(_: *mut ::jvmti::JavaVM,
                                      _: *mut *mut ::std::os::raw::c_void,
                                      _: ::jvmti::jint)
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
