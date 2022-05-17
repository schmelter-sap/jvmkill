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

use std::cmp::max;
use std::collections::HashMap;
use std::io::Write;

pub trait Record {
    fn recordObject(&mut self, class_name: String, object_size: crate::jvmti::jlong);
}

pub trait Print {
    fn print(&self, writer: &mut dyn Write);
}

#[derive(Default)]
struct ObjectStats {
    count: usize,
    total_size: crate::jvmti::jlong,
}

pub struct Stats {
    java_objects: HashMap<String, ObjectStats>,
    max_entries: usize,
}

impl Stats {
    pub fn new(max_entries: usize) -> Stats {
        Stats {
            java_objects: HashMap::new(),
            max_entries,
        }
    }
}

impl Print for Stats {
    fn print(&self, writer: &mut dyn Write) {
        let mut results: Vec<(&String, &ObjectStats)> = self.java_objects.iter().collect();
        results.sort_by(|&(_, s1), &(_, s2)| s2.total_size.cmp(&s1.total_size));

        results.truncate(self.max_entries);

        let max_sig_len = results.iter().map(|&(sig, _)| sig.len()).fold(10, max);

        writeln_paced!(
            writer,
            "| Instance Count | Total Bytes | Class Name{} |",
            " ".repeat(max_sig_len - 10)
        );

        for &(sig, s) in results.iter() {
            writeln_paced!(
                writer,
                "| {:<14} | {:<11} | {}{} |",
                s.count,
                s.total_size,
                sig,
                " ".repeat(max_sig_len - sig.len())
            );
        }
    }
}

impl Record for Stats {
    fn recordObject(&mut self, class_name: String, object_size: crate::jvmti::jlong) {
        let s = self
            .java_objects
            .entry(class_name)
            .or_insert_with(Default::default);
        s.count += 1;
        s.total_size += object_size;
    }
}

#[cfg(test)]
mod tests {
    use super::Print;
    use super::Record;
    use super::Stats;

    #[test]
    fn short_signature() {
        let mut s = Stats::new(100);
        s.recordObject(String::from("aaa"), 20);
        assert_print(
            &s,
            "\
            | Instance Count | Total Bytes | Class Name |\n\
            | 1              | 20          | aaa        |\n",
        );
    }

    #[test]
    fn long_signature() {
        let mut s = Stats::new(100);
        s.recordObject(String::from("abcdefghijklmn"), 20);
        assert_print(
            &s,
            "\
            | Instance Count | Total Bytes | Class Name     |\n\
            | 1              | 20          | abcdefghijklmn |\n",
        );
    }

    #[test]
    fn counting() {
        let mut s = Stats::new(100);
        s.recordObject(String::from("a"), 20);
        s.recordObject(String::from("a"), 15);
        assert_print(
            &s,
            "\
            | Instance Count | Total Bytes | Class Name |\n\
            | 2              | 35          | a          |\n",
        );
    }

    #[test]
    fn sorting() {
        let mut s = Stats::new(100);
        s.recordObject(String::from("b"), 20);
        s.recordObject(String::from("a"), 30);
        s.recordObject(String::from("c"), 10);
        assert_print(
            &s,
            "\
            | Instance Count | Total Bytes | Class Name |\n\
            | 1              | 30          | a          |\n\
            | 1              | 20          | b          |\n\
            | 1              | 10          | c          |\n",
        );
    }

    #[test]
    fn truncation() {
        let mut s = Stats::new(2);
        s.recordObject(String::from("b"), 20);
        s.recordObject(String::from("a"), 30);
        s.recordObject(String::from("c"), 10);
        assert_print(
            &s,
            "\
            | Instance Count | Total Bytes | Class Name |\n\
            | 1              | 30          | a          |\n\
            | 1              | 20          | b          |\n",
        );
    }

    fn assert_print(s: &Stats, expected: &str) {
        let mut buff: Vec<u8> = Vec::new();
        s.print(&mut buff);
        let string_buff = String::from_utf8(buff).expect("invalid UTF-8");
        assert_eq!(string_buff, expected.to_string());
    }
}
