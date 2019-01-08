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
use common::run_java;

mod common;

#[test]
fn memory_time_0_count_0() {
    assert!(!run_java("org.cloudfoundry.jvmkill.MemoryExhaustion", "=printHeapHistogram=1,heapHistogramMaxEntries=20", &["Ljava/lang/Class;", "Heap memory:"], &["jvmkill killing current process"]));
}

#[test]
fn memory_time_10_count_2() {
    assert!(!run_java("org.cloudfoundry.jvmkill.MemoryExhaustion",
                      format!("=time=10,count=2,heapDumpPath={}/dump-%a-%d-%b-%Y-%T-%z.hprof,printHeapHistogram=1,heapHistogramMaxEntries=10", env::temp_dir().to_str().unwrap()).as_str(),
                      &["Heapdump written to"], &["ResourceExhausted! (1/2)", "jvmkill killing current process"])
    );
}
