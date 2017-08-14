/*
 * Copyright (c) 2015-2017 the original author or authors.
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

use common::run_java;

mod common;

#[test]
fn basic() {
    assert!(!run_java("org.cloudfoundry.jvmkill.ThreadExhaustion", "", &[], &["Resource exhaustion event: the JVM was unable to create a thread.",
        "ERROR: PoolStats action failed: cannot determine memory usage statistics since the JVM is unable to create a thread"]));
}

#[test]
fn print_memory_usage_0() {
    assert!(!run_java("org.cloudfoundry.jvmkill.ThreadExhaustion", "=printMemoryUsage=0", &[], &["jvmkill killing current process"]));
}

#[test]
fn time_10_count_2() {
    assert!(!run_java("org.cloudfoundry.jvmkill.ThreadExhaustion", "=time=10,count=2,printHeapHistogram=1,heapHistogramMaxEntries=10,printMemoryUsage=0", &[], &[]));
}

#[test]
fn parallel_time_10_count_2() {
    assert!(!run_java("org.cloudfoundry.jvmkill.ParallelThreadExhaustion", "=time=10,count=2", &[], &[]));
}
