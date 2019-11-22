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

fn main() {
    let i = PathBuf::from(env::var("JAVA_HOME").unwrap()).join("include");
    let p = if cfg!(target_os = "macos") { i.join("darwin") } else { i.join("linux") };

    let bindings = bindgen::Builder::default()
        .header("src/bindings.h")
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg(format!("-I/{}", i.to_str().unwrap()))
        .clang_arg(format!("-I/{}", p.to_str().unwrap()))
        .generate()
        .expect("Failed to generate bindings");

    bindings
        .write_to_file(PathBuf::from("src/bindings.rs"))
        .expect("Failed to write bindings");
}