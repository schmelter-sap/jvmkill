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

use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;
use std::str::Split;

#[derive(Debug, PartialEq)]
pub struct Parameters {
    pub count_threshold: usize,
    pub heap_dump_path: Option<PathBuf>,
    pub heap_histogram_max_entries: usize,
    pub print_heap_histogram: bool,
    pub print_memory_usage: bool,
    pub time_threshold: usize,
}

impl Parameters {
    pub fn new(options: *const c_char) -> Parameters {
        let mut p = Parameters { ..Default::default() };

        if options == ptr::null() {
            return p;
        }

        let s = String::from(unsafe { CStr::from_ptr(options) }
            .to_string_lossy());

        for o in Parameters::parse_options(&s) {
            if o.is_empty() {
                continue;
            }

            let (key, value) = Parameters::parse_option(o);

            match key {
                "count" => p.count_threshold = value.parse().expect("option value must be a number"),
                "heapDumpPath" => p.heap_dump_path = Some(PathBuf::from(value)),
                "heapHistogramMaxEntries" => p.heap_histogram_max_entries = value.parse().expect("option value must be a number"),
                "printHeapHistogram" => p.print_heap_histogram = value.parse::<usize>().expect("option value must be a number") != 0,
                "printMemoryUsage" => p.print_memory_usage = value.parse::<usize>().expect("option value must be a number") != 0,
                "time" => p.time_threshold = value.parse().expect("option value must be a number"),
                _ => assert!(false, "unknown option: {}", key),
            }
        }

        return p;
    }

    fn parse_option(s: &str) -> (&str, &str) {
        let v: Vec<&str> = s.splitn(2, "=").collect();
        assert_eq!(v.len(), 2, "invalid option: {}", s);
        assert!(!v[0].is_empty(), "invalid key: {}", s);
        assert!(!v[1].is_empty(), "invalid value: {}", s);
        return (v[0], v[1]);
    }

    fn parse_options(s: &String) -> Split<char> {
        return s.split(',');
    }
}

impl Default for Parameters {
    fn default() -> Self {
        return Self {
            count_threshold: 0,
            heap_dump_path: None,
            heap_histogram_max_entries: 100,
            print_heap_histogram: false,
            print_memory_usage: true,
            time_threshold: 1,
        };
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::path::PathBuf;

    use crate::context::Parameters;

    #[test]
    fn default_values() {
        let p = Parameters { ..Default::default() };

        assert_eq!(p, Parameters {
            count_threshold: 0,
            heap_dump_path: None,
            heap_histogram_max_entries: 100,
            print_heap_histogram: false,
            print_memory_usage: true,
            time_threshold: 1,
        });
    }

    #[test]
    #[should_panic(expected = "invalid key: =test-value")]
    fn empty_key() {
        create("=test-value");
    }

    #[test]
    #[should_panic(expected = "invalid option: test")]
    fn empty_option() {
        create("test");
    }

    #[test]
    #[should_panic(expected = "invalid value: test-key=")]
    fn empty_value() {
        create("test-key=");
    }

    #[test]
    #[should_panic(expected = "unknown option: test-key")]
    fn invalid_option() {
        create("test-key=test-value");
    }

    #[test]
    fn parses_count() {
        assert_eq!(create("count=42").count_threshold, 42);
    }

    #[test]
    fn parses_heap_dump_path() {
        assert_eq!(create("heapDumpPath=/test").heap_dump_path, Some(PathBuf::from("/test")));
    }

    #[test]
    fn parses_heap_histogram_max_entries() {
        assert_eq!(create("heapHistogramMaxEntries=42").heap_histogram_max_entries, 42);
    }

    #[test]
    fn parses_print_heap_histogram() {
        assert_eq!(create("printHeapHistogram=0").print_heap_histogram, false);
    }

    #[test]
    fn parses_print_memory_usage() {
        assert_eq!(create("printMemoryUsage=0").print_memory_usage, false);
    }

    #[test]
    fn parses_time() {
        assert_eq!(create("time=42").time_threshold, 42);
    }

    fn create(s: &str) -> Parameters {
        let options = CString::new(s)
            .expect("cannot convert to CString");

        return Parameters::new(options.as_ptr());
    }
}
