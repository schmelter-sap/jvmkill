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

use crate::bindings::{jclass, JNI_TRUE, jobject, jvalue};
use crate::jni::JNI;

pub struct HotspotDiagnosticMXBean<'h, J: JNI> {
    class: jclass,
    instance: jobject,
    jni: &'h J,
}

impl<'h, J: JNI> HotspotDiagnosticMXBean<'h, J> {
    pub fn new(class: jclass, instance: jobject, jni: &'h J) -> Self {
        return Self { class, instance, jni };
    }

    pub fn dump_heap(&self, path: &str) {
        let method = self.jni.get_method(self.class, "dumpHeap", "(Ljava/lang/String;Z)V")
            .expect("HotSpotDiagnosticMXBean.dumpHeap not found");

        let p = self.jni.new_string_utf(path);

        self.jni.call_object_method_a(self.instance, method, &[jvalue { l: p }, jvalue { z: JNI_TRUE as u8 }])
            .expect("unable to dump heap");
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use mockall::Sequence;

    use crate::bindings::{jclass, jmethodID, JNI_TRUE, jobject, jstring};
    use crate::jmx::HotspotDiagnosticMXBean;
    use crate::jni::MockJNI;

    #[test]
    fn dump_heap() {
        let mut jni = MockJNI::new();
        let mut seq = Sequence::new();

        let c_hot_spot_diagnostic_mxbean = jni_type!(jclass);
        let i_hot_spot_diagnostic_mxbean = jni_type!(jclass);

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
            .withf_st(move |a_s| a_s == "test-path")
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

        let h = HotspotDiagnosticMXBean::new(c_hot_spot_diagnostic_mxbean, i_hot_spot_diagnostic_mxbean, &jni);
        h.dump_heap("test-path");
    }
}
