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

use std::{env, fs};
use std::path::PathBuf;

use time::OffsetDateTime;

use crate::action;
use crate::action::Action;
use crate::bindings::jint;
use crate::jmx::ManagementFactory;
use crate::jni::JNI;

pub struct HeapDump<'h, J: JNI> {
    factory: &'h ManagementFactory<'h, J>,
    path: PathBuf,
}

impl<'h, J: JNI> HeapDump<'h, J> {
    pub fn new(factory: &'h ManagementFactory<J>, path: &PathBuf) -> Self {
        let mut p = env::current_dir()
            .expect("current directory not found");

        p.push(path);

        return Self { factory, path: p };
    }

    fn create_parent(&self, path: &PathBuf) {
        let p = path.parent()
            .expect("unable to find heap dump parent directory");

        fs::create_dir_all(p)
            .expect("unable to create heap dump parent directory");
    }

    fn resolve_path(&self) -> PathBuf {
        return PathBuf::from(OffsetDateTime::now().format(self.path.to_str().unwrap()));
    }
}

impl<'h, J: JNI> Action for HeapDump<'h, J> {
    fn execute(&self, flags: jint) {
        if action::is_threads_exhausted(flags) {
            eprintln!("cannot create heap dump since the JVM is unable to create a thread");
            return;
        }

        println!("\n>>> Heap Dump");

        let p = self.resolve_path();
        self.create_parent(&p);

        self.factory.get_hotspot_diagnostic_mxbean().dump_heap(p.to_str().unwrap());
        println!("Heap dump written to {:?}", p);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::ptr;

    use mockall::Sequence;

    use crate::action::Action;
    use crate::action::heap_dump::HeapDump;
    use crate::bindings::{jclass, jint, jmethodID, JNI_TRUE, jobject, jstring, JVMTI_RESOURCE_EXHAUSTED_JAVA_HEAP, JVMTI_RESOURCE_EXHAUSTED_THREADS};
    use crate::jmx::ManagementFactory;
    use crate::jni::MockJNI;

    #[test]
    fn execute() {
        let t = PathBuf::from(tempdir::TempDir::new("jvmkill").unwrap().path());
        let u = t.clone();

        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_management_factory = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/ManagementFactory")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_management_factory));

        let m_get_platform_mxbean = jni_type!(jmethodID);
        jni
            .expect_get_static_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_management_factory)
                    && a_method == "getPlatformMXBean"
                    && a_signature == "(Ljava/lang/Class;)Ljava/lang/management/PlatformManagedObject;"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_platform_mxbean));

        let c_hot_spot_diagnostic_mxbean = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "com/sun/management/HotSpotDiagnosticMXBean")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_hot_spot_diagnostic_mxbean));

        let i_hot_spot_diagnostic_mxbean = jni_type!(jobject);
        jni
            .expect_call_static_object_method_a()
            .withf_st(move |&a_class, &a_method, a_args| {
                ptr::eq(c_management_factory, a_class)
                    && ptr::eq(a_method, m_get_platform_mxbean)
                    && ptr::eq(unsafe { a_args[0].l }, c_hot_spot_diagnostic_mxbean)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(i_hot_spot_diagnostic_mxbean));

        let m_dump_heap = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_hot_spot_diagnostic_mxbean)
                    && a_method == "dumpHeap"
                    && a_signature == "(Ljava/lang/String;Z)V"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_dump_heap));

        let s_p = jni_type!(jstring);
        jni
            .expect_new_string_utf()
            .withf_st(move |a_s| a_s == u.to_str().unwrap())
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| s_p);

        let e = jni_type!(jobject);
        jni
            .expect_call_object_method_a()
            .withf_st(move |&a_instance, &a_method, a_args| {
                ptr::eq(a_instance, i_hot_spot_diagnostic_mxbean)
                    && ptr::eq(a_method, m_dump_heap)
                    && ptr::eq(unsafe { a_args[0].l }, s_p)
                    && unsafe { a_args[1].z } == JNI_TRUE as u8
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(e));

        HeapDump::new(&ManagementFactory::new(&jni), &t).execute(JVMTI_RESOURCE_EXHAUSTED_JAVA_HEAP as jint);
    }

    #[test]
    fn execute_threads_exhausted() {
        let t = PathBuf::from(tempdir::TempDir::new("jvmkill").unwrap().path());

        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_management_factory = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/ManagementFactory")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_management_factory));

        HeapDump::new(&ManagementFactory::new(&jni), &t).execute(JVMTI_RESOURCE_EXHAUSTED_THREADS as jint);
    }
}
