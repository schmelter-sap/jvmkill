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

use std::{env, fs};
use std::path::{PathBuf};
use std::process::Command;

pub fn run_java(
    class: &str,
    arguments: &str,
    expected_stdout: &[&str],
    expected_stderr: &[&str],
) -> bool {
    let output = Command::new(&java())
        .arg(format!(
            "-agentpath:{}{}",
            jvmkill().to_str().unwrap(),
            arguments
        ))
        .arg("-cp")
        .arg(jvmkill_classpath())
        .arg("-Xmx50m")
        .arg("-XX:ReservedCodeCacheSize=10m")
        .arg("-XX:-UseCompressedOops")
        .arg(class)
        .output()
        .expect("failed to run Java process");

    assert_contents(&output.stdout, expected_stdout);
    assert_contents(&output.stderr, expected_stderr);

    output.status.success()
}

fn assert_contents(stream: &[u8], expected: &[&str]) {
    let s = String::from_utf8_lossy(stream);
    println!("OUTPUT:\n{}\n:OUTPUT", s);
    let mut success = true;
    for o in expected {
        if !s.contains(o) {
            println!("{}", o);
            success = false;
        }
    }
    if !success {
        println!("the above were not found in:\n{}", s);
    }
    assert!(success);
}

fn java() -> PathBuf {
    PathBuf::from(env::var("JAVA_HOME").unwrap())
        .join("bin")
        .join("java")
}

fn jvmkill_test_dir() -> PathBuf {
    env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("resource-exhaustion-generator")
}

fn jvmkill_test() -> PathBuf {
    jvmkill_test_dir()
        .join("target")
        .join("resource-exhaustion-generator-0.0.0.jar")
}

fn jvmkill_app_dependencies_dir() -> PathBuf {
    jvmkill_test_dir()
        .join("target")
        .join("dependencies")
}

fn jvmkill_app_dependencies_list() -> Vec<PathBuf> {
    fs::read_dir(jvmkill_app_dependencies_dir()).expect("unable to read directory")
        .map(|p| p.expect("unable to read directory").path()).collect()
}

fn jvmkill_classpath() -> String {
    let mut deps = jvmkill_app_dependencies_list();
    deps.push(jvmkill_test());

    deps.iter()
        .map(|p| p.to_str().expect("unable to convert to string"))
        .collect::<Vec<&str>>()
        .join(":")
}

fn jvmkill() -> PathBuf {
    let lib_name = if cfg!(target_os = "macos") {
        "libjvmkill.dylib"
    } else {
        "libjvmkill.so"
    };

    env::current_dir()
        .expect("no current working directory")
        .parent()
        .expect("must have parent")
        .join("target")
        .join("debug")
        .join("deps")
        .join(lib_name)
}
