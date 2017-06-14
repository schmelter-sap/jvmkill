/*
 * Copyright (c) 2017 the original author or authors.
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
use std::io::Write;
use std::io::stdout;
use heap::tagger::Tagger;
use heap::tagger::Tag;
use heap::stats::Stats;
use heap::stats::Record;
use heap::stats::Print;

pub struct HeapHistogram<T: JvmTI + Clone> {
    jvmti: T,
}

impl<T: JvmTI + Clone> HeapHistogram<T> {
    pub fn new(mut jvmti: T) -> Result<Self, ::jvmti::jint> {
        jvmti.enable_object_tagging()?;
        Ok(Self {
            jvmti: jvmti.clone(),
        })
    }

    fn print(&self, writer: &mut Write) {
        let mut tagger = Tagger::new();

        // Tag all loaded classes so we can determine each object's class signature during heap traversal.
        self.jvmti.tag_loaded_classes(&mut tagger);

        let mut heap_stats = Stats::new();

        // Traverse the live heap and add objects to the heap stats.
        self.jvmti.traverse_live_heap(|class_tag: ::jvmti::jlong, size: ::jvmti::jlong| {
            if let Some(sig) = tagger.class_signature(class_tag) {
                heap_stats.recordObject(sig, size);
            }
        });

        heap_stats.print(writer);
    }
}

impl<T: JvmTI + Clone> super::Action for HeapHistogram<T> {
    fn on_oom(&self, _: ::env::JniEnv, _: ::jvmti::jint) {
        self.print(&mut stdout());
    }
}

#[cfg(test)]
mod tests {
    use super::HeapHistogram;
    use ::env::JvmTI;
    use ::env::FnResourceExhausted;
    use std::cell::RefCell;
    use std::sync::Mutex;
    use ::env::RawMonitorId;
    use ::heap::tagger::Tag;

    const test_error_code: ::jvmti::jint = 54;

    #[test]
    fn new_calls_enable_object_tagging() {
        let mockJvmti = MockJvmti::new();
        let hh = HeapHistogram::new(mockJvmti);
        assert!(hh.is_ok());
        assert!((hh.unwrap().jvmti as MockJvmti).object_tagging_enabled);
    }

    #[test]
    fn new_percolates_enable_object_tagging_failure() {
        let mut mockJvmti = MockJvmti::new();
        mockJvmti.object_tagging_enabled_result = Err(test_error_code);
        let hh = HeapHistogram::new(mockJvmti);
        assert!(hh.is_err());
        assert_eq!(hh.err().unwrap(), test_error_code);
    }

    #[test]
    fn print_works() {
        let mockJvmti = MockJvmti::new();
        let hh = HeapHistogram::new(mockJvmti);

        let mut buff: Vec<u8> = Vec::new();
        hh.unwrap().print(&mut buff);
        let string_buff = String::from_utf8(buff).expect("invalid UTF-8");
        assert_eq!(string_buff, "| Instance Count | Total Bytes | Class Name |\n| 2              | 200         | sig2       |\n| 1              | 10          | sig1       |\n".to_string());

    }

    #[derive(Clone, Copy, Default)]
    struct Classes {
        t1: ::jvmti::jlong,
        t2: ::jvmti::jlong
    }

    #[derive(Clone)]
    struct MockJvmti {
        pub object_tagging_enabled_result: Result<(), ::jvmti::jint>,
        pub object_tagging_enabled: bool,
        classes: RefCell<Classes>
    }

    impl MockJvmti {
        fn new() -> MockJvmti {
            MockJvmti {
                object_tagging_enabled_result: Ok(()),
                object_tagging_enabled: false,
                classes: RefCell::new(Default::default())
            }
        }
    }

    impl JvmTI for MockJvmti {
        fn create_raw_monitor(&mut self, name: String, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint> {
            unimplemented!()
        }

        fn raw_monitor_enter(&mut self, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint> {
            unimplemented!()
        }

        fn raw_monitor_exit(&mut self, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint> {
            unimplemented!()
        }

        fn on_resource_exhausted(&mut self, callback: FnResourceExhausted) -> Result<(), ::jvmti::jint> {
            unimplemented!()
        }

        fn enable_object_tagging(&mut self) -> Result<(), ::jvmti::jint> {
            self.object_tagging_enabled = true;
            self.object_tagging_enabled_result
        }

        fn tag_loaded_classes(&self, tagger: &mut Tag) {
            let mut c = self.classes.borrow_mut();
            c.t1 = tagger.class_tag(&"sig1".to_string());
            c.t2 = tagger.class_tag(&"sig2".to_string());
        }

        fn traverse_live_heap<F>(&self, mut closure: F) where F: FnMut(::jvmti::jlong, ::jvmti::jlong) {
            let c = self.classes.borrow();
            closure(c.t1, 10);
            closure(c.t2, 100);
            closure(c.t2, 100);
        }
    }
}
