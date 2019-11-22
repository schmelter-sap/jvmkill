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

use std::fmt::{Display, Error, Formatter};

use crate::action;
use crate::action::Action;
use crate::bindings::jint;
use crate::jmx::{ManagementFactory, MemoryPoolMXBean, MemoryUsage};
use crate::jni::JNI;

pub struct MemoryPools<'m, J: JNI> {
    factory: &'m ManagementFactory<'m, J>
}

impl<'m, J: JNI> MemoryPools<'m, J> {
    pub fn new(factory: &'m ManagementFactory<J>) -> Self {
        return Self { factory };
    }
}

impl<'m, J: JNI> Action for MemoryPools<'m, J> {
    fn execute(&self, flags: jint) {
        if action::is_threads_exhausted(flags) {
            eprintln!("cannot dump memory pools since the JVM is unable to create a thread");
            return;
        }

        println!("\n>>> Memory Pools");

        println!("Memory usage:");
        let m = self.factory.get_memory_mxbean();
        println!("{}", Statistics::from_usage(String::from("Heap memory"), m.get_heap_memory_usage()));
        println!("{}", Statistics::from_usage(String::from("Non-heap memory"), m.get_non_heap_memory_usage()));

        println!("\nMemory pool usage:");
        for p in self.factory.get_memory_pool_mxbeans() {
            println!("{}", Statistics::from_memory_pool(p));
        }
    }
}

struct Statistics {
    committed: i64,
    init: i64,
    max: i64,
    name: String,
    used: i64,
}

impl Statistics {
    fn from_memory_pool<T: JNI>(memory_pool: MemoryPoolMXBean<T>) -> Self {
        return Statistics::from_usage(memory_pool.get_name(), memory_pool.get_usage());
    }

    fn from_usage<T: JNI>(name: String, usage: MemoryUsage<T>) -> Self {
        return Self {
            committed: usage.get_committed(),
            init: usage.get_init(),
            max: usage.get_max(),
            name,
            used: usage.get_used(),
        };
    }

    fn is_near_max(&self) -> bool {
        return (self.committed as f32) / (self.max as f32) >= 0.95;
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut h = "";

        if self.is_near_max() {
            h = match self.name.as_str() {
                "Compressed Class Space" => "\n      Hint: Compressed Class Space is over 95% full. To increase it, set -XX:CompressedClassSpaceSize to a suitable value.",
                "Heap memory" => "\n      Hint: Heap memory is over 95% full. To increase it, increase the container size.",
                "Metaspace" => "\n      Hint: Metaspace is over 95% full. To increase it, set -XX:MaxMetaspaceSize to a suitable value.",
                _ => "",
            };
        }


        return write!(f, "   {}: init {}, used {}, committed {}, max {}{}", self.name, self.init, self.used, self.committed, self.max, h);
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use mockall::Sequence;

    use crate::action::Action;
    use crate::action::memory_pools::MemoryPools;
    use crate::bindings::{jclass, jint, jmethodID, jobject, jstring, JVMTI_RESOURCE_EXHAUSTED_JAVA_HEAP, JVMTI_RESOURCE_EXHAUSTED_THREADS};
    use crate::jmx::ManagementFactory;
    use crate::jni::MockJNI;

    #[test]
    fn execute() {
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

        let m_get_heap_memory_usage = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_memory_mxbean)
                    && a_method == "getHeapMemoryUsage"
                    && a_signature == "()Ljava/lang/management/MemoryUsage;"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_heap_memory_usage));


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
                ptr::eq(a_instance, i_memory_mxbean)
                    && ptr::eq(a_method, m_get_heap_memory_usage)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| Option::Some(i_memory_usage));

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
            .return_once_st(move |_, _| 11);

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
            .return_once_st(move |_, _| 12);

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
            .return_once_st(move |_, _| 13);

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
            .return_once_st(move |_, _| 14);

        let m_get_non_heap_memory_usage = jni_type!(jmethodID);
        jni
            .expect_get_method()
            .withf_st(move |&a_class, a_method, a_signature| {
                ptr::eq(a_class, c_memory_mxbean)
                    && a_method == "getNonHeapMemoryUsage"
                    && a_signature == "()Ljava/lang/management/MemoryUsage;"
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _| Option::Some(m_get_non_heap_memory_usage));

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
                ptr::eq(a_instance, i_memory_mxbean)
                    && ptr::eq(a_method, m_get_non_heap_memory_usage)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| Option::Some(i_memory_usage));

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
            .return_once_st(move |_, _| 21);

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
            .return_once_st(move |_, _| 22);

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
            .return_once_st(move |_, _| 23);

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
            .return_once_st(move |_, _| 24);

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
                ptr::eq(a_instance, i_0)
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
            .return_once(move |_| Option::Some(String::from("test-name-3")));

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
                ptr::eq(a_instance, i_0)
                    && ptr::eq(a_method, m_get_usage)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| Option::Some(i_memory_usage));

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
            .return_once_st(move |_, _| 31);

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
            .return_once_st(move |_, _| 32);

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
            .return_once_st(move |_, _| 33);

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
            .return_once_st(move |_, _| 34);

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
                ptr::eq(a_instance, i_1)
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
            .return_once(move |_| Option::Some(String::from("test-name-4")));

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
                ptr::eq(a_instance, i_1)
                    && ptr::eq(a_method, m_get_usage)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| Option::Some(i_memory_usage));

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
            .return_once_st(move |_, _| 41);

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
            .return_once_st(move |_, _| 43);

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
            .return_once_st(move |_, _| 44);        let m_get_name = jni_type!(jmethodID);
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
                ptr::eq(a_instance, i_2)
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
            .return_once(move |_| Option::Some(String::from("test-name-5")));

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
                ptr::eq(a_instance, i_2)
                    && ptr::eq(a_method, m_get_usage)
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _| Option::Some(i_memory_usage));

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
            .return_once_st(move |_, _| 51);

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
            .return_once_st(move |_, _| 52);

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
            .return_once_st(move |_, _| 53);

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
            .return_once_st(move |_, _| 54);

        MemoryPools::new(&ManagementFactory::new(&jni)).execute(JVMTI_RESOURCE_EXHAUSTED_JAVA_HEAP as jint);
    }

    #[test]
    fn execute_threads_exhausted() {
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_management_factory = jni_type!(jclass);
        jni
            .expect_find_class()
            .withf_st(move |a_class| a_class == "java/lang/management/ManagementFactory")
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_| Option::Some(c_management_factory));

        MemoryPools::new(&ManagementFactory::new(&jni)).execute(JVMTI_RESOURCE_EXHAUSTED_THREADS as jint);
    }
}
