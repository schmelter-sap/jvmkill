/*
 * Copyright (c) 2015-2017 the original author or authors.
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

use std::collections::HashMap;

pub trait Tag {
    fn class_tag(&mut self, sig: &String) -> ::jvmti::jlong;
    fn class_signature(&self, tag: ::jvmti::jlong) -> Option<String>;
}

pub struct Tagger {
    next_class_tag: ::jvmti::jlong,
    sigs: HashMap<::jvmti::jlong, String>,
}

impl Tagger {
    pub fn new() -> Tagger {
        Tagger {
            next_class_tag: 0,
            sigs: HashMap::new(),
        }
    }
}

impl Tag for Tagger {
    fn class_tag(&mut self, sig: &String) -> ::jvmti::jlong {
        self.next_class_tag += 1;
        self.sigs.insert(self.next_class_tag, sig.clone());
        self.next_class_tag
    }

    fn class_signature(&self, tag: ::jvmti::jlong) -> Option<String> {
        self.sigs.get(&tag).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::Tag;

    #[test]
    fn remembers_tagged_classes() {
        let mut t = super::Tagger::new();
        let c = String::from("c");
        let tag_c = t.class_tag(&c);
        let d = String::from("d");
        let tag_d = t.class_tag(&d);

        assert_eq!(c, t.class_signature(tag_c).expect("test error"));
        assert_eq!(d, t.class_signature(tag_d).expect("test error"));
    }
}
