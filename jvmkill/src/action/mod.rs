/*
 * Copyright 2015-2019 the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::action::heap_dump::HeapDump;
use crate::action::heap_histogram::HeapHistogram;
use crate::action::kill::Kill;
use crate::action::memory_pools::MemoryPools;
use crate::action::thread_dump::ThreadDump;
use crate::bindings::{jint, JVMTI_RESOURCE_EXHAUSTED_THREADS};
use crate::context::Parameters;
use crate::jmx::ManagementFactory;
use crate::jni::JNI;
use crate::jvmti::JVMTI;

mod heap_dump;
mod heap_histogram;
mod kill;
mod memory_pools;
mod signal;
mod thread_dump;

pub trait Action {
    fn execute(&self, flags: jint);
}

pub struct Actions<'a> {
    pub actions: Vec<Box<dyn Action + 'a>>
}

impl<'a> Actions<'a> {
    pub fn new<N: JNI, V: JVMTI>(parameters: &Parameters, jvmti: &'a V, factory: &'a ManagementFactory<N>) -> Self {
        let mut actions: Vec<Box<dyn Action>> = Vec::new();

        if parameters.print_heap_histogram {
            actions.push(Box::new(HeapHistogram::new(jvmti, parameters.heap_histogram_max_entries)));
        }

        if parameters.print_memory_usage {
            actions.push(Box::new(MemoryPools::new(factory)));
        }

        actions.push(Box::new(ThreadDump::new()));

        match &parameters.heap_dump_path {
            Some(p) => actions.push(Box::new(HeapDump::new(factory, p))),
            None => {}
        };

        actions.push(Box::new(Kill::new()));

        return Self { actions };
    }

    pub fn execute(&self, flags: jint) {
        for action in &self.actions {
            action.execute(flags);
        }
    }
}

pub fn is_threads_exhausted(flags: jint) -> bool {
    let t = JVMTI_RESOURCE_EXHAUSTED_THREADS as jint;
    return flags & t == t;
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use mockall::Sequence;

    use crate::action::Actions;
    use crate::bindings::jclass;
    use crate::context::Parameters;
    use crate::jmx::ManagementFactory;
    use crate::jni::MockJNI;
    use crate::jvmti::MockJVMTI;

    #[test]
    fn execute() {
        let jvmti = MockJVMTI::new();
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_management_factory = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/ManagementFactory")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_management_factory));

        let factory = ManagementFactory::new(&jni);

        let p = Parameters { ..Default::default() };
        let a = Actions::new(&p, &jvmti, &factory);

        assert_eq!(a.actions.len(), 3);
    }

    #[test]
    fn execute_print_heap_histogram_true() {
        let jvmti = MockJVMTI::new();
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_management_factory = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/ManagementFactory")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_management_factory));

        let factory = ManagementFactory::new(&jni);

        let p = Parameters { print_heap_histogram: true, ..Default::default() };
        let a = Actions::new(&p, &jvmti, &factory);

        assert_eq!(a.actions.len(), 4);
    }

    #[test]
    fn execute_print_memory_usage_false() {
        let jvmti = MockJVMTI::new();
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_management_factory = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/ManagementFactory")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_management_factory));

        let factory = ManagementFactory::new(&jni);

        let p = Parameters { print_memory_usage: false, ..Default::default() };
        let a = Actions::new(&p, &jvmti, &factory);

        assert_eq!(a.actions.len(), 2);
    }

    #[test]
    fn execute_heap_dump_path() {
        let jvmti = MockJVMTI::new();
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_management_factory = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/ManagementFactory")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_management_factory));

        let factory = ManagementFactory::new(&jni);

        let p = Parameters { heap_dump_path: Some(PathBuf::from("test-dir")), ..Default::default() };
        let a = Actions::new(&p, &jvmti, &factory);

        assert_eq!(a.actions.len(), 4);
    }
}
