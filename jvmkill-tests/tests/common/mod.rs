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


use std::env;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
pub struct Runner<'r> {
    pub class: &'r str,
    pub arguments: &'r str,
    pub std_out: Vec<&'r str>,
    pub std_err: Vec<&'r str>,
}

impl<'r> Runner<'r> {
    pub fn run(&self) {
        let o = Command::new(self.java())
            .arg(format!("-agentpath:{}{}", self.agent().to_str().unwrap(), self.arguments))
            .arg("-cp").arg(self.jar())
            .arg("-Xmx50m")
            .arg("-XX:ReservedCodeCacheSize=10m")
            .arg("-XX:-UseCompressedOops")
            .arg(self.class)
            .output()
            .expect("failed to run Java process");

        assert!(!o.status.success());
        self.assert_contents(&o.stdout, &self.std_out);
        self.assert_contents(&o.stderr, &self.std_err);
    }

    fn agent(&self) -> PathBuf {
        let lib_name = if cfg!(target_os = "macos") { "libjvmkill.dylib" } else { "libjvmkill.so" };

        return env::var("LD_LIBRARY_PATH")
            .or(env::var("DYLD_LIBRARY_PATH"))
            .or(env::var("DYLD_FALLBACK_LIBRARY_PATH")).unwrap()
            .split(":")
            .map(|root| PathBuf::from(root).join(lib_name))
            .find(|path| path.exists()).unwrap();
    }

    fn assert_contents(&self, stream: &Vec<u8>, expected: &Vec<&'r str>) {
        let actual = String::from_utf8_lossy(stream);

        assert!(expected.iter().all(|&s| actual.contains(s)),
                "{:?} were not found in:\n>>>\n{}\n<<<\n", expected, actual);
    }

    fn jar(&self) -> PathBuf {
        return env::current_dir().unwrap()
            .parent().unwrap()
            .join("resource-exhaustion-generator").join("target").join("resource-exhaustion-generator-0.0.0.jar");
    }

    fn java(&self) -> PathBuf {
        return PathBuf::from(env::var("JAVA_HOME").unwrap())
            .join("bin").join("java");
    }
}
