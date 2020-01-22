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
use crate::jmx::memory_usage::MemoryUsage;
use crate::jni::JNI;

pub struct MemoryPoolMXBean<'m, J: JNI> {
    class: jclass,
    instance: jobject,
    jni: &'m J,
}

impl<'m, J: JNI> MemoryPoolMXBean<'m, J> {
    pub fn new(class: jclass, instance: jobject, jni: &'m J) -> Self {
        return Self { class, instance, jni };
    }

    pub fn get_name(&self) -> String {
        let method = self.jni.get_method(self.class, "getName", "()Ljava/lang/String;")
            .expect("MemoryPoolMXBean.getName not found");

        let n = self.jni.call_object_method(self.instance, method)
            .expect("unable to get name");

        return self.jni.get_string_utf_chars(n)
            .expect("unable to convert name");
    }

    pub fn get_usage(&self) -> MemoryUsage<J> {
        let method = self.jni.get_method(self.class, "getUsage", "()Ljava/lang/management/MemoryUsage;")
            .expect("MemoryPoolMXBean.getUsage not found");

        let class = self.jni.find_class("java/lang/management/MemoryUsage")
            .expect("MemoryUsage not found");

        let instance = self.jni.call_object_method(self.instance, method)
            .expect("unable to get usage");

        return MemoryUsage::new(class, instance, self.jni);
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use mockall::Sequence;

    use crate::bindings::{jclass, jmethodID, jobject, jstring};
    use crate::jmx::MemoryPoolMXBean;
    use crate::jni::MockJNI;

    #[test]
    fn get_name() {
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_memory_pool_mxbean = jni_type!(jclass);
        let i_memory_pool_mxbean = jni_type!(jobject);

        let m_get_name = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_memory_pool_mxbean)
                    && a_method == "getName"
                    && a_signature == "()Ljava/lang/String;"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_name));

        let s_n = jni_type!(jstring);
        jni
            .expect_call_object_method()
            .withf_st(move |&a_instance, &a_method| {
                ptr::eq(a_instance, i_memory_pool_mxbean)
                    && ptr::eq(a_method, m_get_name)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| Option::Some(s_n));

        jni
            .expect_get_string_utf_chars()
            .withf_st(move |&a_s| ptr::eq(a_s, s_n))
            .times(1)
            .in_sequence(&mut seq)
            .return_once(move |_| Option::Some(String::from("test-name")));

        let m = MemoryPoolMXBean::new(c_memory_pool_mxbean, i_memory_pool_mxbean, &jni);
        assert_eq!(m.get_name(), String::from("test-name"));
    }

    #[test]
    fn get_usage() {
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_memory_pool_mxbean = jni_type!(jclass);
        let i_memory_pool_mxbean = jni_type!(jobject);

        let m_get_usage = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_memory_pool_mxbean)
                    && a_method == "getUsage"
                    && a_signature == "()Ljava/lang/management/MemoryUsage;"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_usage));


        let c_memory_usage = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/MemoryUsage")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_memory_usage));

        let i_memory_usage = jni_type!(jobject);
        jni
            .expect_call_object_method()
            .withf_st(move |&a_instance, &a_method| {
                ptr::eq(a_instance, i_memory_pool_mxbean)
                    && ptr::eq(a_method, m_get_usage)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| Option::Some(i_memory_usage));

        MemoryPoolMXBean::new(c_memory_pool_mxbean, i_memory_pool_mxbean, &jni).get_usage();
    }
}
