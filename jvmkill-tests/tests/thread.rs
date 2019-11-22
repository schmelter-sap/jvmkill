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

use crate::common::Runner;

mod common;

#[test]
fn basic() {
    let r = Runner {
        class: "org.cloudfoundry.jvmkill.ThreadExhaustion",
        arguments: "",
        std_out: vec!(),
        std_err: vec!(
            "cannot dump memory pools since the JVM is unable to create a thread",
            "jvmkill is killing current process",
        ),
    };

    r.run()
}

#[test]
fn print_memory_usage_0() {
    let r = Runner {
        class: "org.cloudfoundry.jvmkill.ThreadExhaustion",
        arguments: "=printMemoryUsage=0",
        std_out: vec!(),
        std_err: vec!("jvmkill is killing current process"),
    };

    r.run()
}

#[test]
fn time_10_count_2() {
    let r = Runner {
        class: "org.cloudfoundry.jvmkill.ThreadExhaustion",
        arguments: "=time=10,count=2,printHeapHistogram=0,printMemoryUsage=0",
        std_out: vec!(),
        std_err: vec!(
            "Resource Exhausted! (1/2)",
            "jvmkill is killing current process",
        ),
    };

    r.run()
}

#[test]
fn parallel_time_10_count_2() {
    let r = Runner {
        class: "org.cloudfoundry.jvmkill.ParallelThreadExhaustion",
        arguments: "=time=10,count=2,printHeapHistogram=0,printMemoryUsage=0",
        std_out: vec!(),
        std_err: vec!("jvmkill is killing current process"),
    };

    r.run()
}
