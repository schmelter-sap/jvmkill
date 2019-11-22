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

use crate::bindings::{jclass, jobject, jvalue};
use crate::jmx::hotspot_diagnostic_mxbean::HotspotDiagnosticMXBean;
use crate::jmx::memory_mxbean::MemoryMXBean;
use crate::jmx::memory_pool_mxbean::MemoryPoolMXBean;
use crate::jni::JNI;

pub struct ManagementFactory<'m, J: JNI> {
    class: jclass,
    jni: &'m J,
}

impl<'m, J: JNI> ManagementFactory<'m, J> {
    pub fn new(jni: &'m J) -> Self {
        let class = jni.find_class("java/lang/management/ManagementFactory")
            .expect("ManagementFactory not found");

        return Self { class, jni };
    }

    pub fn get_hotspot_diagnostic_mxbean(&self) -> HotspotDiagnosticMXBean<J> {
        let method = self.jni.get_static_method(self.class, "getPlatformMXBean", "(Ljava/lang/Class;)Ljava/lang/management/PlatformManagedObject;")
            .expect("ManagementFactory.getPlatformMXBean not found");

        let class = self.jni.find_class("com/sun/management/HotSpotDiagnosticMXBean")
            .expect("HotSpotDiagnosticMXBean not found");

        let instance = self.jni.call_static_object_method_a(self.class, method, &[jvalue { l: class }])
            .expect("unable to get HotSpotDiagnosticMXBean");

        return HotspotDiagnosticMXBean::new(class, instance, &self.jni);
    }

    pub fn get_memory_mxbean(&self) -> MemoryMXBean<J> {
        let method = self.jni.get_static_method(self.class, "getMemoryMXBean", "()Ljava/lang/management/MemoryMXBean;")
            .expect("ManagementFactory.getMemoryMXBean not found");

        let class = self.jni.find_class("java/lang/management/MemoryMXBean")
            .expect("MemoryMXBean not found");

        let instance = self.jni.call_static_object_method_a(self.class, method, &[jvalue { l: class }])
            .expect("unable to get MemoryMXBean");

        return MemoryMXBean::new(class, instance, &self.jni);
    }

    pub fn get_memory_pool_mxbeans(&self) -> Vec<MemoryPoolMXBean<J>> {
        let method = self.jni.get_static_method(self.class, "getMemoryPoolMXBeans", "()Ljava/util/List;")
            .expect("ManagementFactory.getMemoryPoolMXBeans not found");

        let instance = self.jni.call_static_object_method(self.class, method)
            .expect("unable to get MemoryPoolMXBeans");

        return self.to_vec(instance);
    }

    fn size(&self, class: jclass, instance: jobject) -> i32 {
        let method = self.jni.get_method(class, "size", "()I")
            .expect("List.size not found");

        return self.jni.call_int_method(instance, method);
    }

    fn to_vec(&self, instance: jobject) -> Vec<MemoryPoolMXBean<J>> {
        let class = self.jni.find_class("java/util/List")
            .expect("List not found");

        let method = self.jni.get_method(class, "get", "(I)Ljava/lang/Object;")
            .expect("List.get not found");

        let c = self.jni.find_class("java/lang/management/MemoryPoolMXBean")
            .expect("MemoryPoolMXBean not found");

        let mut v = Vec::new();

        for i in 0..self.size(class, instance) {
            let p = self.jni.call_object_method_a(instance, method, &[jvalue { i }])
                .expect("unable to get MemoryPoolMXBean");

            v.push(MemoryPoolMXBean::new(c, p, self.jni));
        }

        return v;
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use mockall::Sequence;

    use crate::bindings::{jclass, jint, jmethodID, jobject};
    use crate::jmx::ManagementFactory;
    use crate::jni::MockJNI;

    #[test]
    fn get_hotspot_diagnostic_mxbean() {
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

        let i_hotspot_diagnostic_mxbean = jni_type!(jobject);
        jni
            .expect_call_static_object_method_a()
            .withf_st(move |&a_class, &a_method, a_args| {
                ptr::eq(a_class, c_management_factory)
                    && ptr::eq(a_method, m_get_platform_mxbean)
                    && ptr::eq(unsafe { a_args[0].l }, c_hot_spot_diagnostic_mxbean)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(i_hotspot_diagnostic_mxbean));

        ManagementFactory::new(&jni).get_hotspot_diagnostic_mxbean();
    }

    #[test]
    fn get_memory_mxbean() {
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_management_factory = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/ManagementFactory")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_management_factory));

        let m_get_memory_mxbean = jni_type!(jmethodID);
        jni
            .expect_get_static_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_management_factory)
                    && a_method == "getMemoryMXBean"
                    && a_signature == "()Ljava/lang/management/MemoryMXBean;"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_memory_mxbean));

        let c_memory_mxbean = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/MemoryMXBean")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_memory_mxbean));

        let i_memory_mxbean = jni_type!(jobject);
        jni
            .expect_call_static_object_method_a()
            .withf_st(move |&a_class, &a_method, a_args| {
                ptr::eq(a_class, c_management_factory)
                    && ptr::eq(a_method, m_get_memory_mxbean)
                    && ptr::eq(unsafe { a_args[0].l }, c_memory_mxbean)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(i_memory_mxbean));

        ManagementFactory::new(&jni).get_memory_mxbean();
    }

    #[test]
    fn get_memory_pool_mxbeans() {
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_management_factory = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/ManagementFactory")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_management_factory));

        let m_get_memory_pool_mxbeans = jni_type!(jmethodID);
        jni
            .expect_get_static_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_management_factory)
                    && a_method == "getMemoryPoolMXBeans"
                    && a_signature == "()Ljava/util/List;"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_memory_pool_mxbeans));

        let i_list = jni_type!(jobject);
        jni
            .expect_call_static_object_method()
            .withf_st(move |&a_class, &a_method| {
                ptr::eq(a_class, c_management_factory)
                    && ptr::eq(a_method, m_get_memory_pool_mxbeans)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| Option::Some(i_list));

        let c_list = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/util/List")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_list));

        let m_get = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_list)
                    && a_method == "get"
                    && a_signature == "(I)Ljava/lang/Object;"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get));

        let c_memory_pool_mxbean = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/MemoryPoolMXBean")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_memory_pool_mxbean));

        let m_size = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_list)
                    && a_method == "size"
                    && a_signature == "()I"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_size));

        jni
            .expect_call_int_method()
            .withf_st(move |&a_instance, &a_method| {
                ptr::eq(a_instance, i_list)
                    && ptr::eq(a_method, m_size)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| 3);

        let i_0 = jni_type!(jobject);
        jni
            .expect_call_object_method_a()
            .withf_st(move |&a_instance, &a_method, a_args| {
                ptr::eq(a_instance, i_list)
                    && ptr::eq(a_method, m_get)
                    && (unsafe { a_args[0].i } == 0 as jint)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(i_0));

        let i_1 = jni_type!(jobject);
        jni
            .expect_call_object_method_a()
            .withf_st(move |&a_instance, &a_method, a_args| {
                ptr::eq(a_instance, i_list)
                    && ptr::eq(a_method, m_get)
                    && (unsafe { a_args[0].i } == 1 as jint)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(i_1));

        let i_2 = jni_type!(jobject);
        jni
            .expect_call_object_method_a()
            .withf_st(move |&a_instance, &a_method, a_args| {
                ptr::eq(a_instance, i_list)
                    && ptr::eq(a_method, m_get)
                    && (unsafe { a_args[0].i } == 2 as jint)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(i_2));

        let m = ManagementFactory::new(&jni);
        let mp = m.get_memory_pool_mxbeans();
        assert_eq!(mp.len(), 3)
    }
}
