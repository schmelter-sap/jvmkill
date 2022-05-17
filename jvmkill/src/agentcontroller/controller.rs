/*
 * Copyright (c) 2015-2022 the original author or authors.
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

use libc::SIGQUIT;
use std::io::Write;

pub struct AgentController<'a, T: Write> {
    heuristic: Box<dyn super::Heuristic + 'a>,
    actions: Vec<Box<dyn super::Action>>,
    log: T,
}

impl<'a, T: Write> AgentController<'a, T> {
    pub fn new(
        ti: crate::env::JvmTiEnv,
        options: *mut ::std::os::raw::c_char,
        log: T,
    ) -> Result<Self, crate::jvmti::jint> {
        let parms = super::parms::AgentParameters::parseParameters(options);

        let mut ac = Self {
            heuristic: Box::new(super::threshold::Threshold::new(
                parms.count_threshold,
                parms.time_threshold,
            )),
            actions: Vec::new(),
            log,
        };

        if parms.print_heap_histogram {
            ac.actions.push(Box::new(
                super::heaphistogram::HeapHistogram::new(ti, parms.heap_histogram_max_entries)
                    .map_err(|err| err.rc())?,
            ));
        }

        if let Some(path) = parms.heap_dump_path {
            ac.actions
                .push(Box::new(super::heapdump::HeapDump::new(path)));
        }

        if parms.print_memory_usage {
            ac.actions
                .push(Box::new(super::poolstats::PoolStats::new()));
        }

        let mut threadDump = super::kill::Kill::new();
        threadDump.setSignal(SIGQUIT);
        ac.actions.push(Box::new(threadDump));

        ac.actions.push(Box::new(super::kill::Kill::new()));

        Ok(ac)
    }

    #[cfg(test)]
    fn test_new(
        heuristic: Box<dyn super::Heuristic + 'a>,
        actions: Vec<Box<dyn super::Action>>,
        log: T,
    ) -> Self {
        Self {
            heuristic,
            actions,
            log,
        }
    }
}

impl<'a, T: Write> super::MutAction for AgentController<'a, T> {
    fn on_oom(&mut self, jni_env: crate::env::JniEnv, resource_exhaustion_flags: crate::jvmti::jint) {
        const oom_error: crate::jvmti::jint =
            crate::jvmti::JVMTI_RESOURCE_EXHAUSTED_OOM_ERROR as crate::jvmti::jint;

        writeln!(
            &mut self.log,
            "\nResource exhaustion event{}.",
            resource_exhaustion_symptom(resource_exhaustion_flags)
        )
        .unwrap();

        if self.heuristic.on_oom() {
            for action in &self.actions {
                if let Err(error) = action.on_oom(jni_env, resource_exhaustion_flags) {
                    writeln!(&mut self.log, "ERROR: {} action failed: {}", action, error).unwrap();
                }
            }
        } else if resource_exhaustion_flags & oom_error == oom_error {
            writeln!(
                &mut self.log,
                "\nThe JVM is about to throw a java.lang.OutOfMemoryError."
            )
            .unwrap();
        }
    }
}

fn resource_exhaustion_symptom(resource_exhaustion_flags: crate::jvmti::jint) -> &'static str {
    const heap_exhausted: crate::jvmti::jint =
        crate::jvmti::JVMTI_RESOURCE_EXHAUSTED_JAVA_HEAP as crate::jvmti::jint;
    const threads_exhausted: crate::jvmti::jint =
        crate::jvmti::JVMTI_RESOURCE_EXHAUSTED_THREADS as crate::jvmti::jint;

    if resource_exhaustion_flags & heap_exhausted == heap_exhausted {
        if resource_exhaustion_flags & threads_exhausted == threads_exhausted {
            ": the JVM was unable to allocate memory from the heap and create a thread"
        } else {
            ": the JVM was unable to allocate memory from the heap"
        }
    } else if resource_exhaustion_flags & threads_exhausted == threads_exhausted {
        ": the JVM was unable to create a thread"
    } else {
        ""
    }
}

unsafe impl<'a, T: Write> Send for AgentController<'a, T> {}

unsafe impl<'a, T: Write> Sync for AgentController<'a, T> {}

#[cfg(test)]
mod tests {
    use crate::agentcontroller::MutAction;

    pub struct TestHeuristic {
        call_count: u32,
    }

    impl TestHeuristic {
        pub fn new() -> Self {
            Self { call_count: 0 }
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

    impl ::std::fmt::Display for TestAction {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "TestAction")
        }
    }

    impl super::super::Action for TestAction {
        fn on_oom(&self, _: crate::env::JniEnv, _: crate::jvmti::jint) -> Result<(), crate::err::Error> {
            panic!("TestAction.on_oom")
        }
    }

    trait Output {
        fn output(&self) -> String;
    }

    impl<'a> Output for super::AgentController<'a, Vec<u8>> {
        fn output(&self) -> String {
            String::from_utf8(self.log.to_vec()).unwrap()
        }
    }

    #[test]
    fn prints_default_resource_exhaustion_message() {
        let mut ac = test_ac();
        ac.on_oom(dummy_jni_env(), 0);
        assert_eq!(ac.output(), "\nResource exhaustion event.\n")
    }

    #[test]
    fn prints_heap_resource_exhaustion_message() {
        let mut ac = test_ac();
        ac.on_oom(
            dummy_jni_env(),
            crate::jvmti::JVMTI_RESOURCE_EXHAUSTED_JAVA_HEAP as crate::jvmti::jint,
        );
        assert_eq!(
            ac.output(),
            "\nResource exhaustion event: the JVM was unable to allocate memory from the heap.\n"
        )
    }

    #[test]
    fn prints_thread_resource_exhaustion_message() {
        let mut ac = test_ac();
        ac.on_oom(
            dummy_jni_env(),
            crate::jvmti::JVMTI_RESOURCE_EXHAUSTED_THREADS as crate::jvmti::jint,
        );
        assert_eq!(
            ac.output(),
            "\nResource exhaustion event: the JVM was unable to create a thread.\n"
        )
    }

    #[test]
    fn prints_heap_and_thread_resource_exhaustion_message() {
        let mut ac = test_ac();
        ac.on_oom(
            dummy_jni_env(),
            (crate::jvmti::JVMTI_RESOURCE_EXHAUSTED_JAVA_HEAP as crate::jvmti::jint)
                + (crate::jvmti::JVMTI_RESOURCE_EXHAUSTED_THREADS) as crate::jvmti::jint,
        );
        assert_eq!(ac.output(), "\nResource exhaustion event: the JVM was unable to allocate memory from the heap and create a thread.\n")
    }

    #[test]
    fn does_not_call_action_when_heuristic_returns_false() {
        let mut ac = test_ac();
        ac.on_oom(dummy_jni_env(), 0);
    }

    #[test]
    #[should_panic(expected = "TestAction.on_oom")]
    fn calls_action_when_heuristic_returns_true() {
        let mut ac = test_ac();
        ac.on_oom(dummy_jni_env(), 0);
        ac.on_oom(dummy_jni_env(), 0);
    }

    fn test_ac<'a>() -> super::AgentController<'a, Vec<u8>> {
        let heuristic = Box::new(TestHeuristic::new());
        super::AgentController::test_new(heuristic, vec![Box::new(TestAction::new())], Vec::new())
    }

    fn dummy_jni_env() -> crate::env::JniEnv {
        crate::env::JniEnv::new(::std::ptr::null_mut())
    }
}
