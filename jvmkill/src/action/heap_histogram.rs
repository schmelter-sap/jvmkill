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

use std::cmp;

use crate::action::Action;
use crate::bindings::{jint, jlong, JNI_TRUE, jvmtiCapabilities};
use crate::heap::{ClassFormatter, Contents, Types};
use crate::jvmti::JVMTI;

pub struct HeapHistogram<'h, J: JVMTI> {
    jvmti: &'h J,
    max_entries: usize,
}

impl<'h, J: JVMTI> HeapHistogram<'h, J> {
    pub fn new(jvmti: &'h J, max_entries: usize) -> Self {
        return Self { jvmti, max_entries };
    }
}

impl<'h, J: JVMTI> Action for HeapHistogram<'h, J> {
    fn execute(&self, _flags: jint) {
        let mut c: jvmtiCapabilities = Default::default();
        c.set_can_tag_objects(JNI_TRUE);
        self.jvmti.add_capabilities(c);

        let mut types = Types::new(self.jvmti);
        types.tag_classes();

        let mut contents = Contents::new(self.jvmti);
        contents.analyze_heap();

        let f = ClassFormatter::new();

        let mut max = 10;
        let formatted: Vec<(usize, jlong, String)> = contents.get_contents(self.max_entries).iter()
            .map(|s| (s.count, s.total_size, f.format(types.get(s.tag))))
            .inspect(|(_c, _s, n)| max = cmp::max(max, n.len()))
            .collect();

        println!("\n>>> Heap Histogram");
        println!("| Instance Count | Total Bytes | Class Name{} |", " ".repeat(max - 10));
        println!("| -------------- | ----------- | {} |", "-".repeat(max));
        for (c, s, n) in formatted {
            println!("| {:<14} | {:<11} | {}{} |", c, s, n, " ".repeat(max - n.len()));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::raw::c_void;
    use std::ptr;

    use mockall::Sequence;

    use crate::action::Action;
    use crate::action::heap_histogram::HeapHistogram;
    use crate::bindings::{jclass, jint, jlong, JNI_TRUE, JVMTI_RESOURCE_EXHAUSTED_JAVA_HEAP, jvmtiHeapCallbacks, jvmtiHeapReferenceInfo, jvmtiHeapReferenceKind_JVMTI_HEAP_REFERENCE_CLASS};
    use crate::jvmti::{ArrayPointerLoadedClassesIterator, MockJVMTI};

    #[test]
    fn execute() {
        let mut jvmti = MockJVMTI::new();
        let mut seq = Sequence::new();

        jvmti
            .expect_add_capabilities()
            .withf_st(|&a_capabilities| a_capabilities.can_tag_objects() == JNI_TRUE)
            .times(1)
            .in_sequence(&mut seq)
            .return_const(());

        let classes = jni_type!(3, jclass) as *mut jclass;
        let loaded_classes = ArrayPointerLoadedClassesIterator { count: 3, classes };
        jvmti
            .expect_get_loaded_classes()
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move || loaded_classes);

        jvmti
            .expect_set_tag()
            .withf_st(move |&a_class, &a_tag| {
                ptr::eq(a_class, classes)
                    && a_tag == 0
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_const(());

        jvmti
            .expect_get_class_signature()
            .withf_st(move |&a_class| ptr::eq(a_class, classes))
            .times(1)
            .in_sequence(&mut seq)
            .return_const((String::from("Lalpha;"), String::from("alpha-generic")));

        jvmti
            .expect_set_tag()
            .withf_st(move |&a_class, &a_tag| {
                ptr::eq(a_class, unsafe { classes.offset(1) })
                    && a_tag == 1
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_const(());

        jvmti
            .expect_get_class_signature()
            .withf_st(move |&a_class| ptr::eq(a_class, unsafe { classes.offset(1) }))
            .times(1)
            .in_sequence(&mut seq)
            .return_const((String::from("Lbravo;"), String::from("bravo-generic")));

        jvmti
            .expect_set_tag()
            .withf_st(move |&a_class, &a_tag| {
                ptr::eq(a_class, unsafe { classes.offset(2) })
                    && a_tag == 2
            })
            .times(1)
            .in_sequence(&mut seq)
            .return_const(());

        jvmti
            .expect_get_class_signature()
            .withf_st(move |&a_class| ptr::eq(a_class, unsafe { classes.offset(2) }))
            .times(1)
            .in_sequence(&mut seq)
            .return_const((String::from("Lcharlie;"), String::from("charlie-generic")));

        let reference_kind = jvmtiHeapReferenceKind_JVMTI_HEAP_REFERENCE_CLASS;
        let reference_info = jni_type_const!(jvmtiHeapReferenceInfo);
        let referrer_class_tag = 100 as jlong;
        let tag_ptr = jni_type!(jlong) as *mut jlong;
        let referrer_tag_ptr = jni_type!(jlong) as *mut jlong;
        let length = 100 as jint;

        jvmti
            .expect_follow_references()
            .times(1)
            .in_sequence(&mut seq)
            .return_once_st(move |_, _, _, c: *const jvmtiHeapCallbacks, u: *const c_void| {
                unsafe {
                    let h = (*c).heap_reference_callback
                        .unwrap();

                    *tag_ptr = 0 << 31;
                    h(reference_kind, reference_info, 0, referrer_class_tag, 10, tag_ptr, referrer_tag_ptr, length, u as *mut c_void);
                    *tag_ptr = 0 << 31;
                    h(reference_kind, reference_info, 0, referrer_class_tag, 10, tag_ptr, referrer_tag_ptr, length, u as *mut c_void);
                    *tag_ptr = 0 << 31;
                    h(reference_kind, reference_info, 1, referrer_class_tag, 20, tag_ptr, referrer_tag_ptr, length, u as *mut c_void);
                    *tag_ptr = 0 << 31;
                    h(reference_kind, reference_info, 1, referrer_class_tag, 20, tag_ptr, referrer_tag_ptr, length, u as *mut c_void);
                    *tag_ptr = 0 << 31;
                    h(reference_kind, reference_info, 2, referrer_class_tag, 30, tag_ptr, referrer_tag_ptr, length, u as *mut c_void);
                    *tag_ptr = 0 << 31;
                    h(reference_kind, reference_info, 2, referrer_class_tag, 30, tag_ptr, referrer_tag_ptr, length, u as *mut c_void);
                }

                return ();
            });

        HeapHistogram::new(&jvmti, 2).execute(JVMTI_RESOURCE_EXHAUSTED_JAVA_HEAP as jint);
    }
}
