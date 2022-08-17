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

use chrono::prelude::*;
use std::env;
use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub struct HeapDump {
    path: PathBuf,
}

impl HeapDump {
    pub fn new(path: PathBuf) -> Self {
        let mut abs_path = env::current_dir().expect("current directory not found");
        abs_path.push(path); // if path is absolute, it replaces abs_path
        abs_path.to_str().expect("heapDumpPath is invalid UTF-8"); // diagnose this problem early
        Self { path: abs_path }
    }
}

impl ::std::fmt::Display for HeapDump {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "HeapDump")
    }
}

impl super::Action for HeapDump {
    fn on_oom(
        &self,
        mut jni_env: crate::env::JniEnv,
        resource_exhaustion_flags: crate::jvmti::jint,
    ) -> Result<(), crate::err::Error> {
        // Do not attempt to generate heapdump on thread exhaustion as this fails abruptly.
        const threads_exhausted: crate::jvmti::jint =
            crate::jvmti::JVMTI_RESOURCE_EXHAUSTED_THREADS as crate::jvmti::jint;
        if resource_exhaustion_flags & threads_exhausted == threads_exhausted {
            return Err(crate::err::Error::ActionUnavailableOnThreadExhaustion(
                "generate a heap dump".to_string(),
            ));
        }

        let mf_class = jni_env.find_class("java/lang/management/ManagementFactory")?;

        if let Ok(hotspot_diagnostic_mxbean_class) =
            jni_env.find_class("com/sun/management/HotSpotDiagnosticMXBean")
        {
            let get_platform_mxbean_method_id = jni_env.get_static_method_id(
                mf_class,
                "getPlatformMXBean",
                "(Ljava/lang/Class;)Ljava/lang/management/PlatformManagedObject;",
            )?;
            let hotspot_diagnostic_mxbean = jni_env.call_static_object_method_with_jclass(
                mf_class,
                get_platform_mxbean_method_id,
                hotspot_diagnostic_mxbean_class,
            )?;
            let dump_heap_method_id = jni_env.get_method_id(
                hotspot_diagnostic_mxbean_class,
                "dumpHeap",
                "(Ljava/lang/String;Z)V",
            )?;

            let resolved_heap_dump_path = resolve_path(&self.path);
            let resolved_heap_dump_path_parent = resolved_heap_dump_path
                .parent()
                .expect("heapDumpPath has no parent directory");

            let resolved_heap_dump_path_cstring = CString::new(
                resolved_heap_dump_path
                    .to_str()
                    .expect("resolved heap dump path contains invalid UTF-8"),
            )
            .expect("invalid resolved heap dump path");

            fs::create_dir_all(&resolved_heap_dump_path_parent).map_err(|err| {
                crate::err::Error::Io(
                    format!(
                        "failed to create heap dump directory {:?}",
                        resolved_heap_dump_path_cstring
                    ),
                    err,
                )
            })?;

            jni_env.call_void_method_with_cstring_jboolean(
                hotspot_diagnostic_mxbean,
                dump_heap_method_id,
                resolved_heap_dump_path_cstring.clone(),
                crate::jvmti::JNI_TRUE as u8,
            )?;

            writeln_paced!(
                &mut ::std::io::stdout(),
                "\nHeapdump written to {:?}",
                resolved_heap_dump_path_cstring
            );
        }

        Ok(())
    }
}

fn resolve_path(path: &Path) -> PathBuf {
    Utc::now()
        .format(path.to_str().expect("heapDumpPath is invalid UTF-8"))
        .to_string()
        .into()
}
