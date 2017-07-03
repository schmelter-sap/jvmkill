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

extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let java_include_directory = PathBuf::from(env::var("JAVA_HOME").unwrap()).join("include");

    let java_platform_include_directory = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };

    let bindings = bindgen::Builder::default()
        .no_unstable_rust()
        .header("jvmti_wrapper.h")
        .clang_arg(clang_include(&java_include_directory))
        .clang_arg(clang_include(&java_include_directory.join(java_platform_include_directory)))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("jvmti_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn clang_include(path: &PathBuf) -> String {
    format!("-I/{}", path.to_str().unwrap())
}
