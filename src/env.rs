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

use std::mem::size_of;
use std::mem::transmute;
use ::std::ptr;
use ::std::ffi::CString;
use ::std::ffi::CStr;
use ::std::sync::Mutex;
use ::jvmti::jvmtiEnv;
use ::jvmti::jrawMonitorID;
use ::heap::tagger::Tag;

pub trait JvmTI {
    // TODO: rework the following methods not to return values since they only ever return Ok(()) or panic.
    fn create_raw_monitor(&mut self, name: String, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint>;
    fn raw_monitor_enter(&mut self, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint>;
    fn raw_monitor_exit(&mut self, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint>;
    fn on_resource_exhausted(&mut self, callback: FnResourceExhausted) -> Result<(), ::jvmti::jint>;
    fn enable_object_tagging(&mut self) -> Result<(), ::jvmti::jint>;
    fn tag_loaded_classes(&self, tagger: &mut Tag);

    // Restriction: traverse_live_heap may be called at most once in the lifetime of a JVM.
    fn traverse_live_heap<F>(&self, closure: F) where F: FnMut(::jvmti::jlong, ::jvmti::jlong);
}

pub struct RawMonitorId {
    id: *mut jrawMonitorID,
}

impl RawMonitorId {
    pub fn new() -> RawMonitorId {
        RawMonitorId {
            id: Box::into_raw(Box::new(ptr::null_mut())),
        }
    }
}

unsafe impl Send for RawMonitorId {}

#[derive(Clone, Copy)]
pub struct JvmTiEnv {
    jvmti: *mut jvmtiEnv
}

impl JvmTiEnv {
    pub fn new(vm: *mut ::jvmti::JavaVM) -> Result<JvmTiEnv, ::jvmti::jint> {
        let mut penv: *mut ::std::os::raw::c_void = ptr::null_mut();
        let rc;
        unsafe {
            rc = (**vm).GetEnv.unwrap()(vm, &mut penv, ::jvmti::JVMTI_VERSION as i32);
        }
        if rc as u32 != ::jvmti::JNI_OK {
            eprintln!("ERROR: GetEnv failed: {}", rc);
            return Err(::jvmti::JNI_ERR);
        }
        Ok(JvmTiEnv { jvmti: penv as *mut jvmtiEnv })
    }

    pub fn wrap(jvmti_env: *mut jvmtiEnv) -> JvmTiEnv {
        JvmTiEnv { jvmti: jvmti_env }
    }
}

macro_rules! jvmtifn (
    ($r:expr, $f:ident, $($arg:tt)*) => { {
        let rc;
        #[allow(unused_unsafe)] // suppress warning if used inside unsafe block
        unsafe {
            let fnc = (**$r).$f.unwrap();
            rc = fnc($r, $($arg)*);
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            eprintln!("ERROR: JVMTI {} failed: {:?}", stringify!($f), rc);
            panic!(::jvmti::JNI_ERR);
        }
    } }
);

// Pick a suitable object tag mask greater than tags used to tag classes.
const TAG_VISITED_MASK: ::jvmti::jlong = 1 << 31;

impl JvmTI for JvmTiEnv {
    fn create_raw_monitor(&mut self, name: String, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint> {
        jvmtifn!(self.jvmti, CreateRawMonitor, CString::new(name).unwrap().into_raw(), monitor.lock().unwrap().id);
        Ok(())
    }

    fn raw_monitor_enter(&mut self, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint> {
        jvmtifn!(self.jvmti, RawMonitorEnter, *monitor.lock().unwrap().id);
        Ok(())
    }

    fn raw_monitor_exit(&mut self, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint> {
        jvmtifn!(self.jvmti, RawMonitorExit, *monitor.lock().unwrap().id);
        Ok(())
    }

    fn on_resource_exhausted(&mut self, callback: FnResourceExhausted) -> Result<(), ::jvmti::jint> {
        unsafe {
            EVENT_CALLBACKS.resource_exhausted = Some(callback);
        }

        let callbacks = ::jvmti::jvmtiEventCallbacks {
            VMInit: None,
            VMDeath: None,
            ThreadStart: None,
            ThreadEnd: None,
            ClassFileLoadHook: None,
            ClassLoad: None,
            ClassPrepare: None,
            VMStart: None,
            Exception: None,
            ExceptionCatch: None,
            SingleStep: None,
            FramePop: None,
            Breakpoint: None,
            FieldAccess: None,
            FieldModification: None,
            MethodEntry: None,
            MethodExit: None,
            NativeMethodBind: None,
            CompiledMethodLoad: None,
            CompiledMethodUnload: None,
            DynamicCodeGenerated: None,
            DataDumpRequest: None,
            reserved72: None,
            MonitorWait: None,
            MonitorWaited: None,
            MonitorContendedEnter: None,
            MonitorContendedEntered: None,
            reserved77: None,
            reserved78: None,
            reserved79: None,
            ResourceExhausted: Some(resource_exhausted),
            GarbageCollectionStart: None,
            GarbageCollectionFinish: None,
            ObjectFree: None,
            VMObjectAlloc: None
        };
        jvmtifn!(self.jvmti, SetEventCallbacks, &callbacks, size_of::<::jvmti::jvmtiEventCallbacks>() as i32);

        jvmtifn!(self.jvmti, SetEventNotificationMode, ::jvmti::jvmtiEventMode::JVMTI_ENABLE, ::jvmti::jvmtiEvent::JVMTI_EVENT_RESOURCE_EXHAUSTED, ::std::ptr::null_mut());

        Ok(())
    }

    fn enable_object_tagging(&mut self) -> Result<(), ::jvmti::jint> {
        let mut capabilities = ::jvmti::jvmtiCapabilities {
            _bitfield_1: [0; 4],
            _bitfield_2: [0; 2],
            _bitfield_3: [0; 2],
            _bitfield_4: [0; 2],
            __bindgen_align: [],
            // FIXME: seems dangeous to reference a field with this name. Same may be true of other fields in this struct.
        };

        jvmtifn!(self.jvmti, GetCapabilities, &mut capabilities);

        capabilities.set_can_tag_objects(1);

        jvmtifn!(self.jvmti, AddCapabilities, &capabilities);

        Ok(())
    }

    fn tag_loaded_classes(&self, tagger: &mut Tag) {
        let mut class_count = 0;
        let mut class_ptr = ::std::ptr::null_mut();
        jvmtifn!(self.jvmti, GetLoadedClasses, &mut class_count, &mut class_ptr);

        while class_count > 0 {
            let mut sig_ptr = ::std::ptr::null_mut();
            jvmtifn!(self.jvmti, GetClassSignature, *class_ptr, &mut sig_ptr, ::std::ptr::null_mut());
            unsafe {
                let cstr = CStr::from_ptr(sig_ptr); // sig_ptr is deallocated later
                let tag = tagger.class_tag(&cstr.to_str().unwrap().to_string());
                jvmtifn!(self.jvmti, SetTag, *class_ptr, tag);
            }
            jvmtifn!(self.jvmti, Deallocate, sig_ptr as *mut u8);

            class_count -= 1;
            unsafe { class_ptr = class_ptr.offset(1); }
        }
    }

    fn traverse_live_heap<F>(&self, mut closure: F)
        where F: FnMut(::jvmti::jlong, ::jvmti::jlong) {
        let callbacks = ::jvmti::jvmtiHeapCallbacks {
            heap_iteration_callback: None,
            heap_reference_callback: Some(heapReferenceCallback),
            primitive_field_callback: None,
            array_primitive_value_callback: None,
            string_primitive_value_callback: None,
            reserved5: None,
            reserved6: None,
            reserved7: None,
            reserved8: None,
            reserved9: None,
            reserved10: None,
            reserved11: None,
            reserved12: None,
            reserved13: None,
            reserved14: None,
            reserved15: None,
        };
        // Pass closure to the callback as a thin pointer pointing to a fat pointer pointing to the closure.
        // See: https://stackoverflow.com/questions/38995701/how-do-i-pass-closures-through-raw-pointers-as-arguments-to-c-functions
        let mut closure_ptr: &mut FnMut(::jvmti::jlong, ::jvmti::jlong) = &mut closure;
        let closure_ptr_ptr = unsafe { transmute(&mut closure_ptr) };

        // Need to pass the traversal state into FollowReferences and pick it up in the callback, which may be called multiple times
        jvmtifn!(self.jvmti, FollowReferences, 0, ::std::ptr::null_mut(), ::std::ptr::null_mut(), &callbacks, closure_ptr_ptr);
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn resource_exhausted(jvmti_env: *mut ::jvmti::jvmtiEnv,
                                        jni_env: *mut ::jvmti::JNIEnv,
                                        flags: ::jvmti::jint,
                                        reserved: *const ::std::os::raw::c_void,
                                        description: *const ::std::os::raw::c_char) -> () {
    match EVENT_CALLBACKS.resource_exhausted {
        Some(function) => {
            let jvmti_env = JvmTiEnv::wrap(jvmti_env);
            function(jvmti_env, JniEnv::new(jni_env), flags);
        }
        None => println!("No resource exhaustion exit registered")
    }
}

pub type FnResourceExhausted = fn(jvmti_env: JvmTiEnv, jni_env: JniEnv, flags: ::jvmti::jint);

#[derive(Default, Clone)]
pub struct EventCallbacks {
    pub resource_exhausted: Option<FnResourceExhausted>
}

pub static mut EVENT_CALLBACKS: EventCallbacks = EventCallbacks {
    resource_exhausted: None
};

#[allow(unused_variables)]
unsafe extern "C" fn heapReferenceCallback(reference_kind: ::jvmti::jvmtiHeapReferenceKind,
                                           reference_info: *const ::jvmti::jvmtiHeapReferenceInfo,
                                           class_tag: ::jvmti::jlong,
                                           referrer_class_tag: ::jvmti::jlong,
                                           size: ::jvmti::jlong,
                                           tag_ptr: *mut ::jvmti::jlong,
                                           referrer_tag_ptr: *mut ::jvmti::jlong,
                                           length: ::jvmti::jint,
                                           user_data: *mut ::std::os::raw::c_void)
                                           -> ::jvmti::jint {
    if *tag_ptr & TAG_VISITED_MASK == TAG_VISITED_MASK {
        return 0;
    }

    // For each object encountered, tag it so we can avoid visiting it again
    // noting that traverse_live_heap is called at most once in the lifetime of a JVM
    *tag_ptr |= TAG_VISITED_MASK;

    // Add the object to the heap stats along with its class signature.
    let unmaskedClassTag = class_tag & !TAG_VISITED_MASK;
    let closure: &mut &mut FnMut(::jvmti::jlong, ::jvmti::jlong) -> ::jvmti::jint = transmute(user_data);
    closure(unmaskedClassTag, size);

    ::jvmti::JVMTI_VISIT_OBJECTS as ::jvmti::jint
}

#[derive(Clone, Copy)]
pub struct JniEnv {
    jni: *mut ::jvmti::JNIEnv
}

impl JniEnv {
    pub fn new(jni_env: *mut ::jvmti::JNIEnv) -> JniEnv {
        JniEnv { jni: jni_env }
    }

    pub fn call_int_method(&mut self, object: ::jvmti::jobject, method_id: ::jvmti::jmethodID) -> ::jvmti::jint {
        unsafe {
            (**self.jni).CallIntMethod.unwrap()(self.jni, object, method_id)
        }
    }

    pub fn call_long_method(&mut self, object: ::jvmti::jobject, method_id: ::jvmti::jmethodID) -> ::jvmti::jlong {
        unsafe {
            (**self.jni).CallLongMethod.unwrap()(self.jni, object, method_id)
        }
    }

    pub fn call_object_method(&mut self, object: ::jvmti::jobject, method_id: ::jvmti::jmethodID) -> Option<::jvmti::jobject> {
        let result;
        unsafe {
            result = (**self.jni).CallObjectMethod.unwrap()(self.jni, object, method_id);
        }

        if result == ptr::null_mut() {
            eprintln!("ERROR: call to method_id {:?} on object {:?} failed", method_id, object);
            None
        } else {
            Some(result)
        }
    }

    // Rust doesn't have variadic functions (except for unsafe FFI bindings).
    pub fn call_object_method_with_int(&mut self, object: ::jvmti::jobject, method_id: ::jvmti::jmethodID, n: ::jvmti::jint) -> Option<::jvmti::jobject> {
        let result;
        unsafe {
            result = (**self.jni).CallObjectMethod.unwrap()(self.jni, object, method_id, n);
        }

        if result == ptr::null_mut() {
            eprintln!("ERROR: call to method_id {:?} on object {:?} with variable argument {} failed", method_id, object, n);
            None
        } else {
            Some(result)
        }
    }

    pub fn call_static_object_method(&mut self, class: ::jvmti::jclass, method_id: ::jvmti::jmethodID) -> Option<::jvmti::jobject> {
        let object;
        unsafe {
            object = (**self.jni).CallStaticObjectMethod.unwrap()(self.jni, class, method_id);
        }

        if object == ptr::null_mut() {
            eprintln!("ERROR: call to method_id {:?} on class {:?} failed", method_id, class);
            None
        } else {
            Some(object)
        }
    }

    pub fn find_class(&mut self, class_name: &str) -> Option<::jvmti::jclass> {
        let class;
        unsafe {
            class = (**self.jni).FindClass.unwrap()(self.jni, CString::new(class_name).unwrap().as_ptr())
        }

        if class == ptr::null_mut() {
            eprintln!("ERROR: {} class not found", class_name);
            None
        } else {
            Some(class)
        }
    }

    pub fn get_method_id(&mut self, class: ::jvmti::jclass, method: &str, signature: &str) -> Option<::jvmti::jmethodID> {
        let method_id;
        unsafe {
            method_id = (**self.jni).GetMethodID.unwrap()(self.jni, class, CString::new(method).unwrap().as_ptr(), CString::new(signature).unwrap().as_ptr());
        }

        if method_id == ptr::null_mut() {
            eprintln!("ERROR: {} method with signature {} not found", method, signature);
            None
        } else {
            Some(method_id)
        }
    }

    pub fn get_object_class(&mut self, object: ::jvmti::jobject) -> Option<::jvmti::jclass> {
        let class;
        unsafe {
            class = (**self.jni).GetObjectClass.unwrap()(self.jni, object)
        }

        if class == ptr::null_mut() {
            eprintln!("ERROR: class for object {:?} not found", object);
            None
        } else {
            Some(class)
        }
    }

    pub fn get_static_method_id(&mut self, class: ::jvmti::jclass, method: &str, signature: &str) -> Option<::jvmti::jmethodID> {
        let method_id;
        unsafe {
            method_id = (**self.jni).GetStaticMethodID.unwrap()(self.jni, class, CString::new(method).unwrap().as_ptr(), CString::new(signature).unwrap().as_ptr());
        }

        if method_id == ptr::null_mut() {
            eprintln!("ERROR: {} static method with signature {} not found", method, signature);
            None
        } else {
            Some(method_id)
        }
    }

    pub fn get_string_utf_chars<'a>(&mut self, s: ::jvmti::jstring) -> (*const ::std::os::raw::c_char, &'a CStr) {
        let utf_chars;
        let cstr;
        unsafe {
            utf_chars = (**self.jni).GetStringUTFChars.unwrap()(self.jni, s, ptr::null_mut());
            cstr = CStr::from_ptr(utf_chars);
        }

        (utf_chars, cstr)
    }

    pub fn release_string_utf_chars(&mut self, s: ::jvmti::jstring, utf_chars: *const ::std::os::raw::c_char) {
        unsafe {
            (**self.jni).ReleaseStringUTFChars.unwrap()(self.jni, s, utf_chars);
        }
    }
}
