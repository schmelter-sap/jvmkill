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

use std::env;
use std::ffi::CString;
use std::fs;
use std::path::PathBuf;
use time::{now, strftime};

pub struct HeapDump {
    path: PathBuf
}

impl HeapDump {
    pub fn new(path: PathBuf) -> Self {
        let mut abs_path = env::current_dir().unwrap();
        abs_path.push(path); // if path is absolute, it replaces abs_path
        Self {
            path: abs_path
        }
    }
}

impl super::Action for HeapDump {
    fn on_oom(&self, mut jni_env: ::env::JniEnv, resource_exhaustion_flags: ::jvmti::jint) {
        // Do not attempt to generate heapdump on thread exhaustion as this fails abruptly.
        const threads_exhausted: ::jvmti::jint = ::jvmti::JVMTI_RESOURCE_EXHAUSTED_THREADS as ::jvmti::jint;
        if resource_exhaustion_flags & threads_exhausted == threads_exhausted {
            eprintln!("\nThe JVM was unable to create a thread. In these circumstances, a heap dump cannot be generated.");
            return;
        }

        let mf_class = jni_env.find_class("java/lang/management/ManagementFactory").unwrap();

        if let Some(hotspot_diagnostic_mxbean_class) = jni_env.find_class("com/sun/management/HotSpotDiagnosticMXBean") {
            let get_platform_mxbean_method_id = jni_env.get_static_method_id(mf_class, "getPlatformMXBean", "(Ljava/lang/Class;)Ljava/lang/management/PlatformManagedObject;").unwrap();
            let hotspot_diagnostic_mxbean = jni_env.call_static_object_method_with_jclass(mf_class, get_platform_mxbean_method_id, hotspot_diagnostic_mxbean_class).unwrap();
            let dump_heap_method_id = jni_env.get_method_id(hotspot_diagnostic_mxbean_class, "dumpHeap", "(Ljava/lang/String;Z)V").unwrap();

            let resolved_heap_dump_path;
            match strftime(self.path.to_str().unwrap(), &now()) {
                Ok(resolved_path) => resolved_heap_dump_path = resolved_path,
                Err(error) => {
                    eprintln!("ERROR: strftime of {} failed: {}", self.path.to_str().unwrap(), error);
                    return;
                }
            }

            if let Err(error) = fs::create_dir_all(&self.path.parent().unwrap()) {
                eprintln!("ERROR: create_dir_all of {} failed: {}", self.path.parent().unwrap().to_str().unwrap(), error);
                return;
            }


            let heapdump_result = jni_env.call_object_method_with_cstring_jboolean(hotspot_diagnostic_mxbean,
                                                                                   dump_heap_method_id,
                                                                                   CString::new(resolved_heap_dump_path.clone()).unwrap(),
                                                                                   ::jvmti::JNI_TRUE as u8);
            if heapdump_result == None {
                jni_env.diagnose_exception(&mut ::std::io::stderr(), "ERROR: dumpHeap method threw an exception: ");
                return;
            }

            writeln_paced!(&mut ::std::io::stdout(), "\nHeapdump written to {}", resolved_heap_dump_path);
        }
    }
}