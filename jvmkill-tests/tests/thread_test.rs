/*
 * Copyright (c) 2015-2018 the original author or authors.
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

#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;
use crate::common::run_java;

mod common;

// Serialise the thread tests to avoid crashing macOS High Sierra
lazy_static! {
    static ref THREAD_TEST_MUTEX: Mutex<()> = Mutex::new(());
}

#[test]
fn thread_basic() {
    let _g = THREAD_TEST_MUTEX.lock().unwrap();
    assert!(!run_java("org.cloudfoundry.jvmkill.ThreadExhaustion", "", &[], &["Resource exhaustion event: the JVM was unable to create a thread.",
        "ERROR: PoolStats action failed"]));
}

#[test]
fn thread_print_memory_usage_0() {
    let _g = THREAD_TEST_MUTEX.lock().unwrap();
    assert!(!run_java("org.cloudfoundry.jvmkill.ThreadExhaustion", "=printMemoryUsage=0", &[], &["jvmkill killing current process"]));
}

#[test]
fn thread_time_10_count_2() {
    let _g = THREAD_TEST_MUTEX.lock().unwrap();
    assert!(!run_java("org.cloudfoundry.jvmkill.ThreadExhaustion", "=time=10,count=2,printHeapHistogram=0,printMemoryUsage=0",
                      &[], &["ResourceExhausted! (1/2)", "jvmkill killing current process"]));
}

#[test]
fn thread_parallel_time_10_count_2() {
    let _g = THREAD_TEST_MUTEX.lock().unwrap();
    assert!(!run_java("org.cloudfoundry.jvmkill.ParallelThreadExhaustion", "=time=10,count=2,printHeapHistogram=0,printMemoryUsage=0", &[],
                      &["jvmkill killing current process"]));
}
