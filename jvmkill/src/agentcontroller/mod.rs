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

use std::fmt::Display;

pub mod controller;

pub trait MutAction {
    // See https://docs.oracle.com/javase/8/docs/platform/jvmti/jvmti.html#jvmtiResourceExhaustionFlags
    fn on_oom(&mut self, jni_env: ::env::JniEnv, resource_exhaustion_flags: ::jvmti::jint);
}

mod heapdump;
mod heaphistogram;
mod kill;
mod threshold;
mod parms;
mod poolstats;

trait Heuristic {
    fn on_oom(&mut self) -> bool;
}

trait Action: Display {
    // See https://docs.oracle.com/javase/8/docs/platform/jvmti/jvmti.html#jvmtiResourceExhaustionFlags
    fn on_oom(&self, jni_env: ::env::JniEnv, resource_exhaustion_flags: ::jvmti::jint) -> Result<(), ::err::Error>;
}
