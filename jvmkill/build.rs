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

extern crate bindgen;

use std::env;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .derive_default(true)
        .header(jvmti_wrapper())
        .clang_arg(clang_include(&java_include()))
        .clang_arg(clang_include(&java_include_platform()))
        .generate()
        .expect("Failed to generate bindings");

    bindings
        .write_to_file(jvmti_bindings())
        .expect("Failed to write bindings");
}

fn clang_include(path: &Path) -> String {
    format!("-I/{}", path.to_str().unwrap())
}

fn java_include() -> PathBuf {
    PathBuf::from(env::var("JAVA_HOME").unwrap()).join("include")
}

fn java_include_platform() -> PathBuf {
    if cfg!(target_os = "macos") {
        java_include().join("darwin")
    } else {
        java_include().join("linux")
    }
}

fn jvmti_bindings() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap()).join("jvmti_bindings.rs")
}

fn jvmti_wrapper() -> String {
    String::from("include/jvmti_wrapper.h")
}
