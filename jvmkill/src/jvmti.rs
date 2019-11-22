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

use std::ffi::CStr;
use std::mem;
use std::os::raw::{c_uchar, c_void};
use std::ptr;

use mockall::automock;

use crate::bindings::{JavaVM, jclass, jint, jlong, JNI_OK, jthread, JVMTI_VERSION_1, JVMTI_VERSION_11, JVMTI_VERSION_1_0, JVMTI_VERSION_1_1, JVMTI_VERSION_1_2, JVMTI_VERSION_9, jvmtiCapabilities, jvmtiEnv, jvmtiError_JVMTI_ERROR_NONE, jvmtiEvent, jvmtiEventCallbacks, jvmtiEventMode, jvmtiHeapCallbacks};

#[cfg_attr(test, automock(type LoadedClassesIterator = ArrayPointerLoadedClassesIterator;))]
pub trait JVMTI {
    type LoadedClassesIterator: Iterator<Item=*mut jclass>;

    fn add_capabilities(&self, capabilities: jvmtiCapabilities);

    fn follow_references(&self, heap_filter: jint, class: jclass, initial_object: jclass, callbacks: *const jvmtiHeapCallbacks, user_data: *const c_void);

    fn get_class_signature(&self, class: *mut jclass) -> (String, String);

    fn get_loaded_classes(&self) -> Self::LoadedClassesIterator;

    fn set_event_callbacks(&self, callbacks: *const jvmtiEventCallbacks);

    fn set_event_notification_mode(&self, mode: jvmtiEventMode, event_type: jvmtiEvent, event_thread: jthread);

    fn set_tag(&self, class: *mut jclass, tag: jlong);
}

pub struct DefaultJVMTI {
    internal: *mut jvmtiEnv,
}

impl DefaultJVMTI {
    pub fn new(jvmti_env: *mut jvmtiEnv) -> Self {
        return Self { internal: jvmti_env };
    }

    pub fn from(vm: *mut JavaVM) -> Self {
        let p = Self::get_jvmti(vm);
        return Self::new(p as *mut jvmtiEnv);
    }

    fn deallocate(&self, mem: *mut c_uchar) {
        let f = unsafe { (**self.internal).Deallocate }
            .expect("jvmtiEnv.Deallocate not found");

        let r = unsafe { f(self.internal, mem) };
        if r != jvmtiError_JVMTI_ERROR_NONE {
            panic!("unable to deallocate: {}", r);
        }
    }

    fn get_jvmti(vm: *mut JavaVM) -> *mut c_void {
        let f = unsafe { (**vm).GetEnv }
            .expect("jvmtiEnv.GetEnv method not found");

        for c in vec!(JVMTI_VERSION_11, JVMTI_VERSION_9, JVMTI_VERSION_1_2, JVMTI_VERSION_1_1, JVMTI_VERSION_1_0, JVMTI_VERSION_1) {
            let mut p = ptr::null_mut();
            let r = unsafe { f(vm, &mut p, c as jint) };
            if r == JNI_OK as i32 {
                return p;
            }
        }

        panic!("JVMTI not available");
    }
}

impl JVMTI for DefaultJVMTI {
    type LoadedClassesIterator = ArrayPointerLoadedClassesIterator;

    fn add_capabilities(&self, capabilities: jvmtiCapabilities) {
        let f = unsafe { (**self.internal).AddCapabilities }
            .expect("jvmtiEnv.AddCapabilities not found");

        let r = unsafe { f(self.internal, &capabilities) };
        if r != jvmtiError_JVMTI_ERROR_NONE {
            panic!("unable to add callbacks: {}", r);
        }
    }

    fn follow_references(&self, heap_filter: jint, class: jclass, initial_object: jclass, callbacks: *const jvmtiHeapCallbacks, user_data: *const c_void) {
        let f = unsafe { (**self.internal).FollowReferences }
            .expect("jvmtiEnv.FollowReferences not found");

        let r = unsafe { f(self.internal, heap_filter, class, initial_object, callbacks, user_data) };
        if r != jvmtiError_JVMTI_ERROR_NONE {
            panic!("unable to follow references: {}", r);
        }
    }

    fn get_class_signature(&self, class: *mut jclass) -> (String, String) {
        let mut signature = ptr::null_mut();
        let mut generic = ptr::null_mut();

        let f = unsafe { (**self.internal).GetClassSignature }
            .expect("jvmtiEnv.GetClassSignature not found");

        let r = unsafe { f(self.internal, *class, &mut signature, &mut generic) };
        if r != jvmtiError_JVMTI_ERROR_NONE {
            panic!("unable to get class signature: {}", r);
        }

        let s = String::from(unsafe { CStr::from_ptr(signature) }
            .to_string_lossy());

        self.deallocate(signature as *mut c_uchar);

        if generic == ptr::null_mut() {
            return (s, String::new());
        }

        let g = String::from(unsafe { CStr::from_ptr(generic) }
            .to_string_lossy());

        return (s, g);
    }

    fn get_loaded_classes(&self) -> Self::LoadedClassesIterator {
        let f = unsafe { (**self.internal).GetLoadedClasses }
            .expect("jvmtiEnv.GetLoadedClasses not found");

        let mut count = 0;
        let mut classes = ptr::null_mut();

        let r = unsafe { f(self.internal, &mut count, &mut classes) };
        if r != jvmtiError_JVMTI_ERROR_NONE {
            panic!("unable to get loaded classes: {}", r);
        }

        return ArrayPointerLoadedClassesIterator { count, classes };
    }

    fn set_event_callbacks(&self, callbacks: *const jvmtiEventCallbacks) {
        let f = unsafe { (**self.internal).SetEventCallbacks }
            .expect("jvmtiEnv.SetEventCallbacks method not found");

        let r = unsafe { f(self.internal, callbacks, mem::size_of::<jvmtiEventCallbacks>() as jint) };
        if r != jvmtiError_JVMTI_ERROR_NONE {
            panic!("unable to set event callbacks: {}", r);
        }
    }

    fn set_event_notification_mode(&self, mode: jvmtiEventMode, event_type: jvmtiEvent, event_thread: jthread) {
        let f = unsafe { (**self.internal).SetEventNotificationMode }
            .expect("jvmti.SetEventNotificationMode method not found");

        let r = unsafe { f(self.internal, mode, event_type, event_thread) };
        if r != jvmtiError_JVMTI_ERROR_NONE {
            panic!("unable to set event notification mode: {}", r);
        }
    }

    fn set_tag(&self, class: *mut jclass, tag: jlong) {
        let f = unsafe { (**self.internal).SetTag }
            .expect("jvmtiEnv.SetTag not found");

        let r = unsafe { f(self.internal, *class, tag) };
        if r != jvmtiError_JVMTI_ERROR_NONE {
            panic!("unable to set tag: {}", r);
        }
    }
}

pub struct ArrayPointerLoadedClassesIterator {
    pub count: i32,
    pub classes: *mut jclass,
}

impl Default for ArrayPointerLoadedClassesIterator {
    fn default() -> Self {
        return Self { count: 0, classes: ptr::null_mut() };
    }
}

impl Iterator for ArrayPointerLoadedClassesIterator {
    type Item = *mut jclass;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }

        let r = Some(self.classes);

        self.count -= 1;
        self.classes = unsafe { self.classes.offset(1) };

        return r;
    }
}
