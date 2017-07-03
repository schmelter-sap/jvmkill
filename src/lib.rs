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

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::sync::Mutex;

#[macro_use]
mod macros;
mod env;
mod heap;
mod jvmti;
mod agentcontroller;

use env::JvmTI;
use agentcontroller::MutAction;

#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate time;

lazy_static! {
    static ref STATIC_CONTEXT: Mutex<AgentContext<'static>> = Mutex::new(AgentContext::new());
    static ref RAW_MONITOR_ID: Mutex<env::RawMonitorId> = Mutex::new(env::RawMonitorId::new());
}

#[derive(Default)]
struct AgentContext<'a> {
    ac: Option<agentcontroller::controller::AgentController<'a>>
}

impl<'a> AgentContext<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set(&mut self, a: agentcontroller::controller::AgentController<'a>) {
        self.ac = Some(a);
    }

    pub fn on_oom(&mut self, jni_env: ::env::JniEnv, resource_exhaustion_flags: ::jvmti::jint) {
        self.ac.as_mut().map(|mut a| a.on_oom(jni_env, resource_exhaustion_flags));
    }
}

#[no_mangle]
#[allow(unused_variables)]
pub extern fn Agent_OnLoad(vm: *mut jvmti::JavaVM, options: *mut ::std::os::raw::c_char,
                           reserved: *mut ::std::os::raw::c_void) -> jvmti::jint {
    let jvmti_env = env::JvmTiEnv::new(vm);

    if let Err(e) = jvmti_env
        .and_then(|ti| agentcontroller::controller::AgentController::new(ti, options))
        .map(|ac| STATIC_CONTEXT.lock().unwrap().set(ac)) {
        return e;
    }

    if let Err(e) = jvmti_env
        .and_then(|mut ti| {
            ti.on_resource_exhausted(resource_exhausted).unwrap();
            ti.create_raw_monitor(String::from("jvmkill"), &RAW_MONITOR_ID)
        }) {
        return e;
    }

    0
}

fn resource_exhausted(mut jvmti_env: env::JvmTiEnv, jni_env: env::JniEnv, flags: ::jvmti::jint) {
    STATIC_CONTEXT.lock().map(|mut a| {
        if let Err(err) = jvmti_env.raw_monitor_enter(&RAW_MONITOR_ID) {
            eprintln!("ERROR: RawMonitorEnter failed: {}", err);
            return;
        }

        a.on_oom(jni_env, flags);

        if let Err(err) = jvmti_env.raw_monitor_exit(&RAW_MONITOR_ID) {
            eprintln!("ERROR: RawMonitorExit failed: {}", err);
            return;
        }
    }).unwrap();
}
