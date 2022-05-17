/*
 * Copyright (c) 2015-2022 the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use env::JvmTI;
use heap::stats::Print;
use heap::stats::Record;
use heap::stats::Stats;
use heap::tagger::Tag;
use heap::tagger::Tagger;
use std::io::stdout;
use std::io::Write;

pub struct HeapHistogram<T: JvmTI + Clone> {
    jvmti: T,
    max_entries: usize,
}

impl<T: JvmTI + Clone> ::std::fmt::Display for HeapHistogram<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "HeapHistogram")
    }
}

impl<T: JvmTI + Clone> HeapHistogram<T> {
    pub fn new(mut jvmti: T, max_entries: usize) -> Result<Self, ::err::Error> {
        jvmti.enable_object_tagging()?;
        Ok(Self {
            jvmti: jvmti.clone(),
            max_entries,
        })
    }

    fn print(&self, writer: &mut dyn Write) -> Result<(), ::err::Error> {
        let mut tagger = Tagger::new();

        // Tag all loaded classes so we can determine each object's class signature during heap traversal.
        self.jvmti.tag_loaded_classes(&mut tagger)?;

        let mut heap_stats = Stats::new(self.max_entries);

        // Traverse the live heap and add objects to the heap stats.
        self.jvmti
            .traverse_live_heap(|class_tag: ::jvmti::jlong, size: ::jvmti::jlong| {
                if let Some(sig) = tagger.class_signature(class_tag) {
                    heap_stats.recordObject(sig, size);
                }
            })?;

        heap_stats.print(writer);
        Ok(())
    }
}

impl<T: JvmTI + Clone> super::Action for HeapHistogram<T> {
    fn on_oom(&self, _: ::env::JniEnv, _: ::jvmti::jint) -> Result<(), ::err::Error> {
        self.print(&mut stdout())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::HeapHistogram;
    use env::FnResourceExhausted;
    use env::JvmTI;
    use heap::tagger::Tag;
    use std::cell::RefCell;

    const test_error_code: ::jvmti::jint = 54;

    #[test]
    fn new_calls_enable_object_tagging() {
        let mockJvmti = MockJvmti::new();
        let hh = HeapHistogram::new(mockJvmti, 100);
        assert!(hh.is_ok());
        assert!((hh.expect("unexpected error").jvmti as MockJvmti).object_tagging_enabled);
    }

    #[test]
    fn new_percolates_enable_object_tagging_failure() {
        let mut mockJvmti = MockJvmti::new();
        mockJvmti.object_tagging_enabled_result = test_error_code;
        let hh = HeapHistogram::new(mockJvmti, 100);
        assert!(hh.is_err());
        match hh.err().expect("unexpected error") {
            ::err::Error::JvmTi(msg, rc) => {
                assert_eq!(msg, "test error".to_string());
                assert_eq!(rc, test_error_code);
            }
            _ => panic!("wrong error value"),
        }
    }

    #[test]
    fn print_works() {
        let mockJvmti = MockJvmti::new();
        let hh = HeapHistogram::new(mockJvmti, 100);

        let mut buff: Vec<u8> = Vec::new();
        hh.expect("invalid HeapHistogram")
            .print(&mut buff)
            .expect("print failed");
        let string_buff = String::from_utf8(buff).expect("invalid UTF-8");
        assert_eq!(string_buff, "| Instance Count | Total Bytes | Class Name |\n| 2              | 200         | sig2       |\n| 1              | 10          | sig1       |\n".to_string());
    }

    #[derive(Clone, Copy, Default)]
    struct Classes {
        t1: ::jvmti::jlong,
        t2: ::jvmti::jlong,
    }

    #[derive(Clone)]
    struct MockJvmti {
        pub object_tagging_enabled_result: ::jvmti::jint,
        pub object_tagging_enabled: bool,
        classes: RefCell<Classes>,
    }

    impl MockJvmti {
        fn new() -> MockJvmti {
            MockJvmti {
                object_tagging_enabled_result: 0,
                object_tagging_enabled: false,
                classes: RefCell::new(Default::default()),
            }
        }
    }

    impl JvmTI for MockJvmti {
        fn on_resource_exhausted(&mut self, _: FnResourceExhausted) -> Result<(), ::err::Error> {
            unimplemented!()
        }

        fn enable_object_tagging(&mut self) -> Result<(), ::err::Error> {
            self.object_tagging_enabled = true;
            if self.object_tagging_enabled_result == 0 {
                Ok(())
            } else {
                Err(::err::Error::JvmTi(
                    "test error".to_string(),
                    self.object_tagging_enabled_result,
                ))
            }
        }

        fn tag_loaded_classes(&self, tagger: &mut dyn Tag) -> Result<(), ::err::Error> {
            let mut c = self.classes.borrow_mut();
            c.t1 = tagger.class_tag("sig1");
            c.t2 = tagger.class_tag("sig2");
            Ok(())
        }

        fn traverse_live_heap<F>(&self, mut closure: F) -> Result<(), ::err::Error>
        where
            F: FnMut(::jvmti::jlong, ::jvmti::jlong),
        {
            let c = self.classes.borrow();
            closure(c.t1, 10);
            closure(c.t2, 100);
            closure(c.t2, 100);
            Ok(())
        }
    }
}
