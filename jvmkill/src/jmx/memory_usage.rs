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

use crate::bindings::{jclass, jobject};
use crate::jni::JNI;

pub struct MemoryUsage<'m, J: JNI> {
    class: jclass,
    instance: jobject,
    jni: &'m J,
}

impl<'m, J: JNI> MemoryUsage<'m, J> {
    pub fn new(class: jclass, instance: jobject, jni: &'m J) -> Self {
        return Self { class, instance, jni };
    }

    pub fn get_committed(&self) -> i64 {
        let method = self.jni.get_method(self.class, "getCommitted", "()J")
            .expect("MemoryUsage.getCommitted not found");

        return self.jni.call_long_method(self.instance, method);
    }

    pub fn get_init(&self) -> i64 {
        let method = self.jni.get_method(self.class, "getInit", "()J")
            .expect("MemoryUsage.getInit not found");

        return self.jni.call_long_method(self.instance, method);
    }

    pub fn get_max(&self) -> i64 {
        let method = self.jni.get_method(self.class, "getMax", "()J")
            .expect("MemoryUsage.getMax not found");

        return self.jni.call_long_method(self.instance, method);
    }

    pub fn get_used(&self) -> i64 {
        let method = self.jni.get_method(self.class, "getUsed", "()J")
            .expect("MemoryUsage.getUsed not found");

        return self.jni.call_long_method(self.instance, method);
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use mockall::Sequence;

    use crate::bindings::{jclass, jmethodID, jobject};
    use crate::jmx::MemoryUsage;
    use crate::jni::MockJNI;

    #[test]
    fn get_committed() {
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_memory_usage = jni_type!(jclass);
        let i_memory_usage = jni_type!(jobject);

        let m_get_committed = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_memory_usage)
                    && a_method == "getCommitted"
                    && a_signature == "()J"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_committed));

        jni
            .expect_call_long_method()
            .withf_st(move |&a_instance, &a_method| {
                ptr::eq(a_instance, i_memory_usage)
                    && ptr::eq(a_method, m_get_committed)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| 42);

        let m = MemoryUsage::new(c_memory_usage, i_memory_usage, &jni);
        assert_eq!(m.get_committed(), 42);
    }

    #[test]
    fn get_init() {
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_memory_usage = jni_type!(jclass);
        let i_memory_usage = jni_type!(jobject);

        let m_get_init = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_memory_usage)
                    && a_method == "getInit"
                    && a_signature == "()J"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_init));

        jni
            .expect_call_long_method()
            .withf_st(move |&a_instance, &a_method| {
                ptr::eq(a_instance, i_memory_usage)
                    && ptr::eq(a_method, m_get_init)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| 42);

        let m = MemoryUsage::new(c_memory_usage, i_memory_usage, &jni);
        assert_eq!(m.get_init(), 42);
    }

    #[test]
    fn get_max() {
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_memory_usage = jni_type!(jclass);
        let i_memory_usage = jni_type!(jobject);

        let m_get_max = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_memory_usage)
                    && a_method == "getMax"
                    && a_signature == "()J"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_max));

        jni
            .expect_call_long_method()
            .withf_st(move |&a_instance, &a_method| {
                ptr::eq(a_instance, i_memory_usage)
                    && ptr::eq(a_method, m_get_max)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| 42);

        let m = MemoryUsage::new(c_memory_usage, i_memory_usage, &jni);
        assert_eq!(m.get_max(), 42);
    }

    #[test]
    fn get_used() {
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_memory_usage = jni_type!(jclass);
        let i_memory_usage = jni_type!(jobject);

        let m_get_used = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_memory_usage)
                    && a_method == "getUsed"
                    && a_signature == "()J"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_used));

        jni
            .expect_call_long_method()
            .withf_st(move |&a_instance, &a_method| {
                ptr::eq(a_instance, i_memory_usage)
                    && ptr::eq(a_method, m_get_used)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| 42);

        let m = MemoryUsage::new(c_memory_usage, i_memory_usage, &jni);
        assert_eq!(m.get_used(), 42);
    }
}
