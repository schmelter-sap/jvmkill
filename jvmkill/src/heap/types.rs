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

use crate::bindings::jlong;
use crate::jvmti::JVMTI;

pub struct Types<'t, J: JVMTI> {
    jvmti: &'t J,
    types: Vec<String>,
}

impl<'t, J: JVMTI> Types<'t, J> {
    pub fn new(jvmti: &'t J) -> Self {
        return Self { jvmti, types: Vec::new() };
    }

    pub fn get(&self, tag: jlong) -> &String {
        return &self.types[tag as usize];
    }

    pub fn tag_classes(&mut self) {
        for c in self.jvmti.get_loaded_classes() {
            self.jvmti.set_tag(c, self.types.len() as jlong);

            let (signature, _) = self.jvmti.get_class_signature(c);
            self.types.push(signature);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use mockall::Sequence;

    use crate::bindings::jclass;
    use crate::heap::Types;
    use crate::jvmti::{ArrayPointerLoadedClassesIterator, MockJVMTI};

    #[test]
    fn tag_classes_and_get() {
        let mut jvmti = MockJVMTI::new();
        let mut seq = Sequence::new();

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
            .return_const((String::from("alpha-type"), String::from("alpha-generic")));

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
            .return_const((String::from("bravo-type"), String::from("bravo-generic")));

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
            .return_const((String::from("charlie-type"), String::from("charlie-generic")));

        let mut t = Types::new(&mut jvmti);
        t.tag_classes();

        assert_eq!(t.get(0), "alpha-type");
        assert_eq!(t.get(1), "bravo-type");
        assert_eq!(t.get(2), "charlie-type");
    }
}
