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

use std::{mem, ptr};
use std::collections::HashMap;
use std::os::raw::c_void;

use crate::bindings::{jint, jlong, JVMTI_VISIT_OBJECTS, jvmtiHeapCallbacks, jvmtiHeapReferenceInfo, jvmtiHeapReferenceKind};
use crate::jvmti::JVMTI;

pub struct Contents<'c, J: JVMTI> {
    jvmti: &'c J,
    contents: Vec<Statistics>,
}

impl<'c, J: JVMTI> Contents<'c, J> {
    pub fn new(jvmti: &'c J) -> Self {
        return Self { jvmti, contents: Vec::new() };
    }

    pub fn analyze_heap(&mut self) {
        let mut contents: HashMap<jlong, Statistics> = HashMap::new();

        let mut c = |tag, size| {
            let s = contents.entry(tag).or_insert(Statistics { tag, ..Default::default() });
            s.count += 1;
            s.total_size += size;
        };

        let mut p: &mut dyn FnMut(jlong, jlong) = &mut c;
        let pp: *const c_void = unsafe { mem::transmute(&mut p) };

        let callbacks = jvmtiHeapCallbacks { heap_reference_callback: Some(heapReferenceCallback), ..Default::default() };
        self.jvmti.follow_references(0, ptr::null_mut(), ptr::null_mut(), &callbacks, pp);

        self.contents = contents.values()
            .cloned()
            .collect();

        self.contents.sort_unstable_by(|s1, s2| s2.total_size.cmp(&s1.total_size));
    }

    pub fn get_contents(&self, limit: usize) -> Vec<Statistics> {
        let mut c = self.contents.to_vec();
        c.truncate(limit);
        return c;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Statistics {
    pub count: usize,
    pub total_size: jlong,
    pub tag: jlong,
}

const TAG_VISITED_MASK: jlong = 1 << 31;

#[allow(non_snake_case)]
unsafe extern "C" fn heapReferenceCallback(_reference_kind: jvmtiHeapReferenceKind, _reference_info: *const jvmtiHeapReferenceInfo, class_tag: jlong, _referrer_class_tag: jlong, size: jlong,
                                           tag_ptr: *mut jlong, _referrer_tag_ptr: *mut jlong, _length: jint, user_data: *mut c_void) -> jint {
    if *tag_ptr & TAG_VISITED_MASK == TAG_VISITED_MASK {
        return 0;
    }

    *tag_ptr |= TAG_VISITED_MASK;

    let tag = class_tag & !TAG_VISITED_MASK;

    let c: &mut &mut dyn FnMut(jlong, jlong) = mem::transmute(user_data);
    c(tag, size);

    return JVMTI_VISIT_OBJECTS as jint;
}

#[cfg(test)]
mod tests {
    use std::os::raw::c_void;

    use mockall::Sequence;

    use crate::bindings::{jint, jlong, jvmtiHeapCallbacks, jvmtiHeapReferenceInfo, jvmtiHeapReferenceKind_JVMTI_HEAP_REFERENCE_CLASS};
    use crate::heap::Contents;
    use crate::heap::contents::Statistics;
    use crate::jvmti::MockJVMTI;

    #[test]
    fn analyze_heap_and_get_contents() {
        let mut jvmti = MockJVMTI::new();
        let mut seq = Sequence::new();

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

        let mut c = Contents::new(&jvmti);
        c.analyze_heap();

        assert_eq!(c.get_contents(2), vec![Statistics { count: 2, total_size: 60, tag: 2 }, Statistics { count: 2, total_size: 40, tag: 1 }])
    }
}
