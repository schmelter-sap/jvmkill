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

use regex::Regex;

pub struct ClassFormatter {
    pattern: Regex
}

impl ClassFormatter {
    pub fn new() -> Self {
        let pattern = Regex::new(r"(\[*)([BCDFIJLSZ])(?:([a-zA-z/$0-9]+);)?").unwrap();
        return Self { pattern };
    }

    pub fn format(&self, raw: &String) -> String {
        let c = self.pattern.captures(raw.as_str())
            .expect(format!("invalid class name: {}", raw).as_str());

        let mut s = String::new();

        match &c[2] {
            "Z" => s.push_str("boolean"),
            "B" => s.push_str("byte"),
            "C" => s.push_str("char"),
            "D" => s.push_str("double"),
            "F" => s.push_str("float"),
            "I" => s.push_str("int"),
            "J" => s.push_str("long"),
            "S" => s.push_str("short"),
            "L" => s.push_str(&c[3].replace("/", ".")),
            _ => panic!("unknown type"),
        };

        for _ in 0..c[1].len() {
            s.push_str("[]");
        }

        return s;
    }
}

#[cfg(test)]
mod tests {
    use crate::heap::ClassFormatter;

    #[test]
    fn arrays() {
        assert_eq!(ClassFormatter::new().format(&String::from("[[Z")), "boolean[][]");
    }

    #[test]
    fn boolean() {
        assert_eq!(ClassFormatter::new().format(&String::from("Z")), "boolean");
    }

    #[test]
    fn byte() {
        assert_eq!(ClassFormatter::new().format(&String::from("B")), "byte");
    }

    #[test]
    fn char() {
        assert_eq!(ClassFormatter::new().format(&String::from("C")), "char");
    }

    #[test]
    fn class() {
        assert_eq!(ClassFormatter::new().format(&String::from("Lorg/cloudfoundry/MyClass;")), "org.cloudfoundry.MyClass");
    }

    #[test]
    fn double() {
        assert_eq!(ClassFormatter::new().format(&String::from("D")), "double");
    }

    #[test]
    fn float() {
        assert_eq!(ClassFormatter::new().format(&String::from("F")), "float");
    }

    #[test]
    fn int() {
        assert_eq!(ClassFormatter::new().format(&String::from("I")), "int");
    }

    #[test]
    #[should_panic(expected = "invalid class name")]
    fn invalid() {
        ClassFormatter::new().format(&String::from("Q"));
    }

    #[test]
    fn long() {
        assert_eq!(ClassFormatter::new().format(&String::from("J")), "long");
    }

    #[test]
    fn short() {
        assert_eq!(ClassFormatter::new().format(&String::from("S")), "short");
    }
}
