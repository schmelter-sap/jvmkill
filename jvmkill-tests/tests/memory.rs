/*
 * Copyright 2015-2020 the original author or authors.
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

use crate::common::Runner;

mod common;

#[test]
fn time_0_count_0() {
    let r = Runner {
        class: "org.cloudfoundry.jvmkill.MemoryExhaustion",
        arguments: "=printHeapHistogram=1,heapHistogramMaxEntries=20",
        std_out: vec!(
            "java.lang.Class",
            "Heap memory:",
        ),
        std_err: vec!("jvmkill is killing current process"),
    };

    r.run()
}

#[test]
fn time_10_count_2() {
    let a = format!("=time=10,count=2,heapDumpPath={}dump-%a-%d-%b-%Y-%T-%z.hprof,printHeapHistogram=1,heapHistogramMaxEntries=10", env::temp_dir().to_str().unwrap());

    let r = Runner {
        class: "org.cloudfoundry.jvmkill.MemoryExhaustion",
        arguments: a.as_str(),
        std_out: vec!("Heap dump written to"),
        std_err: vec!(
            "Resource Exhausted! (1/2)",
            "jvmkill is killing current process",
        ),
    };

    r.run()
}
