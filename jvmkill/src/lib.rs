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

use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::Mutex;

use crate::action::Actions;
use crate::bindings::{JavaVM, jint, JNIEnv, jvmtiEnv, jvmtiEvent_JVMTI_EVENT_RESOURCE_EXHAUSTED, jvmtiEventCallbacks, jvmtiEventMode_JVMTI_ENABLE};
use crate::context::Context;
use crate::jmx::ManagementFactory;
use crate::jni::DefaultJNI;
use crate::jvmti::{DefaultJVMTI, JVMTI};

#[cfg_attr(test, macro_use)]
mod test_macros;

mod action;
mod context;
mod heap;
mod jmx;
mod jni;
mod jvmti;

mod bindings {
    #![allow(dead_code, non_camel_case_types, non_snake_case, non_upper_case_globals)]
    include!("bindings.rs");
}

static mut CONTEXT: Option<Mutex<Context>> = None;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Agent_OnLoad(vm: *mut JavaVM, options: *mut c_char, _reserved: *mut c_void) -> jint {
    Some(Mutex::new(Context::new(options)));

    unsafe {
        CONTEXT = Some(Mutex::new(Context::new(options)))
    }

    let j = DefaultJVMTI::from(vm);
    j.set_event_notification_mode(jvmtiEventMode_JVMTI_ENABLE, jvmtiEvent_JVMTI_EVENT_RESOURCE_EXHAUSTED, ptr::null_mut());
    j.set_event_callbacks(&jvmtiEventCallbacks { ResourceExhausted: Some(resource_exhausted), ..Default::default() });

    return 0;
}

unsafe extern "C" fn resource_exhausted(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, flags: jint, _reserved: *const c_void, _description: *const c_char) {
    match &CONTEXT {
        None => panic!("context not yet set"),
        Some(m) => {
            let mut c = m.lock().unwrap();
            if c.record() {
                let jvmti = DefaultJVMTI::new(jvmti_env);
                let jni = DefaultJNI::new(jni_env);
                let factory = ManagementFactory::new(&jni);

                Actions::new(&c.parameters, &jvmti, &factory).execute(flags);
            }
        }
    }
}
