/*
 * Copyright (c) 2015-2022 the original author or authors.
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

use crate::heap::tagger::Tag;
use crate::jvmti::jvmtiEnv;
use std::ffi::CStr;
use std::ffi::CString;
use std::mem::size_of;
use std::mem::transmute;
use std::ptr;

pub trait JvmTI {
    fn on_resource_exhausted(&mut self, callback: FnResourceExhausted) -> Result<(), crate::err::Error>;
    fn enable_object_tagging(&mut self) -> Result<(), crate::err::Error>;
    fn tag_loaded_classes(&self, tagger: &mut dyn Tag) -> Result<(), crate::err::Error>;

    // Restriction: traverse_live_heap may be called at most once in the lifetime of a JVM.
    fn traverse_live_heap<F>(&self, closure: F) -> Result<(), crate::err::Error>
    where
        F: FnMut(crate::jvmti::jlong, crate::jvmti::jlong);
}

#[derive(Clone, Copy)]
pub struct JvmTiEnv {
    jvmti: *mut jvmtiEnv,
}

impl JvmTiEnv {
    pub fn new(vm: *mut crate::jvmti::JavaVM) -> Result<JvmTiEnv, crate::jvmti::jint> {
        let mut penv: *mut ::std::os::raw::c_void = ptr::null_mut();
        let rc;
        unsafe {
            rc = (**vm).GetEnv.expect("GetEnv function not found")(
                vm,
                &mut penv,
                crate::jvmti::JVMTI_VERSION as i32,
            );
        }
        if rc as u32 != crate::jvmti::JNI_OK {
            eprintln!("ERROR: GetEnv failed: {}", rc);
            return Err(crate::jvmti::JNI_ERR);
        }
        Ok(JvmTiEnv {
            jvmti: penv as *mut jvmtiEnv,
        })
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
            let fnc = (**$r).$f.expect(&format!("{} function not found", stringify!($f)));
            rc = fnc($r, $($arg)*);
        }
        if rc != crate::jvmti::jvmtiError_JVMTI_ERROR_NONE {
            let message = format!("JVMTI {} failed", stringify!($f));
            Err(crate::err::Error::JvmTi(message, rc as i32))
        } else {
            Ok(())
        }
    } }
);

// Pick a suitable object tag mask greater than tags used to tag classes.
const TAG_VISITED_MASK: crate::jvmti::jlong = 1 << 31;

impl JvmTI for JvmTiEnv {
    fn on_resource_exhausted(&mut self, callback: FnResourceExhausted) -> Result<(), crate::err::Error> {
        unsafe {
            EVENT_CALLBACKS.resource_exhausted = Some(callback);
        }

        let callbacks = crate::jvmti::jvmtiEventCallbacks {
            ResourceExhausted: Some(resource_exhausted),
            ..Default::default()
        };
        jvmtifn!(
            self.jvmti,
            SetEventCallbacks,
            &callbacks,
            size_of::<crate::jvmti::jvmtiEventCallbacks>() as i32
        )?;
        jvmtifn!(
            self.jvmti,
            SetEventNotificationMode,
            crate::jvmti::jvmtiEventMode_JVMTI_ENABLE,
            crate::jvmti::jvmtiEvent_JVMTI_EVENT_RESOURCE_EXHAUSTED,
            ::std::ptr::null_mut()
        )?;

        Ok(())
    }

    fn enable_object_tagging(&mut self) -> Result<(), crate::err::Error> {
        let mut capabilities: crate::jvmti::jvmtiCapabilities = Default::default();

        jvmtifn!(self.jvmti, GetCapabilities, &mut capabilities)?;

        capabilities.set_can_tag_objects(1);

        jvmtifn!(self.jvmti, AddCapabilities, &capabilities)?;

        Ok(())
    }

    fn tag_loaded_classes(&self, tagger: &mut dyn Tag) -> Result<(), crate::err::Error> {
        let mut class_count = 0;
        let mut class_ptr = ::std::ptr::null_mut();
        jvmtifn!(
            self.jvmti,
            GetLoadedClasses,
            &mut class_count,
            &mut class_ptr
        )?;

        while class_count > 0 {
            let mut sig_ptr = ::std::ptr::null_mut();
            jvmtifn!(
                self.jvmti,
                GetClassSignature,
                *class_ptr,
                &mut sig_ptr,
                ::std::ptr::null_mut()
            )?;
            unsafe {
                let cstr = CStr::from_ptr(sig_ptr); // sig_ptr is deallocated later
                let tag = tagger.class_tag(cstr.to_str().expect("invalid UTF-8 string"));
                jvmtifn!(self.jvmti, SetTag, *class_ptr, tag)?;
            }
            jvmtifn!(self.jvmti, Deallocate, sig_ptr as *mut u8)?;

            class_count -= 1;
            unsafe {
                class_ptr = class_ptr.offset(1);
            }
        }

        Ok(())
    }

    fn traverse_live_heap<F>(&self, mut closure: F) -> Result<(), crate::err::Error>
    where
        F: FnMut(crate::jvmti::jlong, crate::jvmti::jlong),
    {
        let callbacks = crate::jvmti::jvmtiHeapCallbacks {
            heap_reference_callback: Some(heapReferenceCallback),
            ..Default::default()
        };
        // Pass closure to the callback as a thin pointer pointing to a fat pointer pointing to the closure.
        // See: https://stackoverflow.com/questions/38995701/how-do-i-pass-closures-through-raw-pointers-as-arguments-to-c-functions
        let mut closure_ptr: &mut dyn FnMut(crate::jvmti::jlong, crate::jvmti::jlong) = &mut closure;
        let closure_ptr_ptr = unsafe { transmute(&mut closure_ptr) };

        // Need to pass the traversal state into FollowReferences and pick it up in the callback, which may be called multiple times
        jvmtifn!(
            self.jvmti,
            FollowReferences,
            0,
            ::std::ptr::null_mut(),
            ::std::ptr::null_mut(),
            &callbacks,
            closure_ptr_ptr
        )?;

        Ok(())
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn resource_exhausted(
    jvmti_env: *mut crate::jvmti::jvmtiEnv,
    jni_env: *mut crate::jvmti::JNIEnv,
    flags: crate::jvmti::jint,
    reserved: *const ::std::os::raw::c_void,
    description: *const ::std::os::raw::c_char,
) {
    match EVENT_CALLBACKS.resource_exhausted {
        Some(function) => {
            let jvmti_env = JvmTiEnv::wrap(jvmti_env);
            function(jvmti_env, JniEnv::new(jni_env), flags);
        }
        None => println!("No resource exhaustion exit registered"),
    }
}

pub type FnResourceExhausted = fn(jvmti_env: JvmTiEnv, jni_env: JniEnv, flags: crate::jvmti::jint);

#[derive(Default, Clone)]
pub struct EventCallbacks {
    pub resource_exhausted: Option<FnResourceExhausted>,
}

pub static mut EVENT_CALLBACKS: EventCallbacks = EventCallbacks {
    resource_exhausted: None,
};

#[allow(unused_variables)]
unsafe extern "C" fn heapReferenceCallback(
    reference_kind: crate::jvmti::jvmtiHeapReferenceKind,
    reference_info: *const crate::jvmti::jvmtiHeapReferenceInfo,
    class_tag: crate::jvmti::jlong,
    referrer_class_tag: crate::jvmti::jlong,
    size: crate::jvmti::jlong,
    tag_ptr: *mut crate::jvmti::jlong,
    referrer_tag_ptr: *mut crate::jvmti::jlong,
    length: crate::jvmti::jint,
    user_data: *mut ::std::os::raw::c_void,
) -> crate::jvmti::jint {
    if *tag_ptr & TAG_VISITED_MASK == TAG_VISITED_MASK {
        return 0;
    }

    // For each object encountered, tag it so we can avoid visiting it again
    // noting that traverse_live_heap is called at most once in the lifetime of a JVM
    *tag_ptr |= TAG_VISITED_MASK;

    // Add the object to the heap stats along with its class signature.
    let unmaskedClassTag = class_tag & !TAG_VISITED_MASK;
    let closure =
        &mut *(user_data as *mut &mut dyn FnMut(crate::jvmti::jlong, crate::jvmti::jlong) -> crate::jvmti::jint);
    closure(unmaskedClassTag, size);

    crate::jvmti::JVMTI_VISIT_OBJECTS as crate::jvmti::jint
}

#[derive(Clone, Copy)]
pub struct JniEnv {
    jni: *mut crate::jvmti::JNIEnv,
}

impl JniEnv {
    pub fn new(jni_env: *mut crate::jvmti::JNIEnv) -> JniEnv {
        JniEnv { jni: jni_env }
    }

    pub fn call_int_method(
        &mut self,
        object: crate::jvmti::jobject,
        method_id: crate::jvmti::jmethodID,
    ) -> crate::jvmti::jint {
        unsafe {
            (**self.jni)
                .CallIntMethod
                .expect("CallIntMethod function not found")(self.jni, object, method_id)
        }
    }

    pub fn call_long_method(
        &mut self,
        object: crate::jvmti::jobject,
        method_id: crate::jvmti::jmethodID,
    ) -> crate::jvmti::jlong {
        unsafe {
            (**self.jni)
                .CallLongMethod
                .expect("CallLongMethod function not found")(self.jni, object, method_id)
        }
    }

    pub fn call_object_method(
        &mut self,
        object: crate::jvmti::jobject,
        method_id: crate::jvmti::jmethodID,
    ) -> Result<crate::jvmti::jobject, crate::err::Error> {
        let result = self.call_object_method_internal(object, method_id);
        if self.exception_occurred() || result == None {
            let message = format!(
                "call to method_id {:?} on object {:?} failed",
                method_id, object
            );
            self.diagnose_exception(&message)?;
            return Err(crate::err::Error::Jni(message));
        }

        Ok(result.expect("unexpected error"))
    }

    pub fn call_object_method_internal(
        &mut self,
        object: crate::jvmti::jobject,
        method_id: crate::jvmti::jmethodID,
    ) -> Option<crate::jvmti::jobject> {
        let result;
        unsafe {
            result = (**self.jni)
                .CallObjectMethod
                .expect("CallObjectMethod function not found")(
                self.jni, object, method_id
            );
        }
        if result.is_null() {
            None
        } else {
            Some(result)
        }
    }

    // Rust doesn't have variadic functions (except for unsafe FFI bindings).
    pub fn call_object_method_with_int(
        &mut self,
        object: crate::jvmti::jobject,
        method_id: crate::jvmti::jmethodID,
        n: crate::jvmti::jint,
    ) -> Result<crate::jvmti::jobject, crate::err::Error> {
        let n_value: crate::jvmti::jvalue = crate::jvmti::jvalue { i: n };
        let result;
        unsafe {
            result = (**self.jni)
                .CallObjectMethodA
                .expect("CallObjectMethodA function not found")(
                self.jni, object, method_id, &n_value,
            );
        }
        if self.exception_occurred() || result.is_null() {
            let message = format!(
                "call to method_id {:?} on object {:?} with variable argument {} failed",
                method_id, object, n
            );
            self.diagnose_exception(&message)?;
            Err(crate::err::Error::Jni(message))
        } else {
            Ok(result)
        }
    }

    // Rust doesn't have variadic functions (except for unsafe FFI bindings).
    pub fn call_object_method_with_cstring_jboolean(
        &mut self,
        object: crate::jvmti::jobject,
        method_id: crate::jvmti::jmethodID,
        s: CString,
        b: crate::jvmti::jboolean,
    ) -> Result<crate::jvmti::jobject, crate::err::Error> {
        let result;
        unsafe {
            let s_jstring =
                (**self.jni)
                    .NewStringUTF
                    .expect("NewStringUTF function not found")(self.jni, s.as_ptr());
            let args: [crate::jvmti::jvalue; 2] =
                [crate::jvmti::jvalue { l: s_jstring }, crate::jvmti::jvalue { z: b }];
            result = (**self.jni)
                .CallObjectMethodA
                .expect("CallObjectMethodA function not found")(
                self.jni, object, method_id, &args[0],
            );
        }
        if self.exception_occurred() || result.is_null() {
            let message = format!(
                "call to method_id {:?} on object {:?} with variable arguments {:?}, {} failed",
                method_id, object, s, b
            );
            self.diagnose_exception(&message)?;
            Err(crate::err::Error::Jni(message))
        } else {
            Ok(result)
        }
    }

    pub fn call_static_object_method(
        &mut self,
        class: crate::jvmti::jclass,
        method_id: crate::jvmti::jmethodID,
    ) -> Result<crate::jvmti::jobject, crate::err::Error> {
        let object;
        unsafe {
            object = (**self.jni)
                .CallStaticObjectMethod
                .expect("CallStaticObjectMethod function not found")(
                self.jni, class, method_id
            );
        }
        if self.exception_occurred() || object.is_null() {
            let message = format!(
                "call to method_id {:?} on class {:?} failed",
                method_id, class
            );
            self.diagnose_exception(&message)?;
            Err(crate::err::Error::Jni(message))
        } else {
            Ok(object)
        }
    }

    pub fn call_static_object_method_with_jclass(
        &mut self,
        class: crate::jvmti::jclass,
        method_id: crate::jvmti::jmethodID,
        c: crate::jvmti::jclass,
    ) -> Result<crate::jvmti::jobject, crate::err::Error> {
        let c_value: crate::jvmti::jvalue = crate::jvmti::jvalue { l: c };
        let object;
        unsafe {
            object = (**self.jni)
                .CallStaticObjectMethodA
                .expect("CallStaticObjectMethodA function not found")(
                self.jni, class, method_id, &c_value,
            );
        }
        if self.exception_occurred() || object.is_null() {
            let message = format!(
                "call to method_id {:?} on class {:?} with variable argument {:?} failed",
                method_id, class, c
            );
            self.diagnose_exception(&message)?;
            Err(crate::err::Error::Jni(message))
        } else {
            Ok(object)
        }
    }

    pub fn diagnose_exception(&mut self, message: &str) -> Result<(), crate::err::Error> {
        if !self.exception_occurred() {
            return Ok(());
        }
        let exc;
        unsafe {
            exc = (**self.jni)
                .ExceptionOccurred
                .expect("ExceptionOccurred function not found")(self.jni);
        }
        let exc_class = self
            .get_object_class_internal(exc)
            .expect("exception class not found");
        let get_message_method_id = self
            .get_method_id_internal(exc_class, "getMessage", "()Ljava/lang/String;")
            .expect("exception getMessage method not found");
        let exc_message =
            self.call_object_method_internal(exc, get_message_method_id)
                .expect("Failed to get exception message") as crate::jvmti::jstring;

        let (exc_message_utf_chars, exc_message_cstr) = self.get_string_utf_chars(exc_message);
        let err = Err(crate::err::Error::Jni(format!(
            "{}: {}",
            message,
            exc_message_cstr.to_string_lossy().into_owned()
        )));
        self.release_string_utf_chars(exc_message, exc_message_utf_chars);
        unsafe {
            (**self.jni)
                .ExceptionClear
                .expect("ExceptionClear function not found")(self.jni);
        }
        err
    }

    fn exception_occurred(&mut self) -> bool {
        unsafe {
            (**self.jni)
                .ExceptionCheck
                .expect("ExceptionCheck function not found")(self.jni)
                == crate::jvmti::JNI_TRUE as u8
        }
    }

    pub fn find_class(&mut self, class_name: &str) -> Result<crate::jvmti::jclass, crate::err::Error> {
        let class_name_cstr = CString::new(class_name).expect("invalid class name");
        let class;

        unsafe {
            class = (**self.jni)
                .FindClass
                .expect("FindClass function not found")(
                self.jni, class_name_cstr.as_ptr()
            )
        }

        if self.exception_occurred() || class.is_null() {
            let message = format!("{} class not found", class_name);
            self.diagnose_exception(&message)?;
            Err(crate::err::Error::Jni(message))
        } else {
            Ok(class)
        }
    }

    pub fn get_method_id(
        &mut self,
        class: crate::jvmti::jclass,
        method: &str,
        signature: &str,
    ) -> Result<crate::jvmti::jmethodID, crate::err::Error> {
        let method_id = self.get_method_id_internal(class, method, signature);
        if self.exception_occurred() || method_id == None {
            let message = format!("{} method with signature {} not found", method, signature);
            self.diagnose_exception(&message)?;
            return Err(crate::err::Error::Jni(message));
        }

        Ok(method_id.expect("unexpected error"))
    }

    fn get_method_id_internal(
        &mut self,
        class: crate::jvmti::jclass,
        method: &str,
        signature: &str,
    ) -> Option<crate::jvmti::jmethodID> {
        let method_name = CString::new(method).expect("invalid method name");
        let sig_name = CString::new(signature).expect("invalid method signature");
        let method_id;
        unsafe {
            method_id = (**self.jni)
                .GetMethodID
                .expect("GetMethodID function not found")(
                self.jni,
                class,
                method_name.as_ptr(),
                sig_name.as_ptr(),
            );
        }
        if method_id.is_null() {
            None
        } else {
            Some(method_id)
        }
    }

    pub fn get_object_class(
        &mut self,
        object: crate::jvmti::jobject,
    ) -> Result<crate::jvmti::jclass, crate::err::Error> {
        let class = self.get_object_class_internal(object);
        if self.exception_occurred() || class == None {
            let message = format!("class for object {:?} not found", object);
            self.diagnose_exception(&message)?;
            Err(crate::err::Error::Jni(message))
        } else {
            Ok(class.expect("unexpected error"))
        }
    }

    fn get_object_class_internal(&mut self, object: crate::jvmti::jobject) -> Option<crate::jvmti::jclass> {
        let class;
        unsafe {
            class = (**self.jni)
                .GetObjectClass
                .expect("GetObjectClass function not found")(self.jni, object)
        }
        if class.is_null() {
            None
        } else {
            Some(class)
        }
    }

    pub fn get_static_method_id(
        &mut self,
        class: crate::jvmti::jclass,
        method: &str,
        signature: &str,
    ) -> Result<crate::jvmti::jmethodID, crate::err::Error> {
        let method_id;
        let method_name = CString::new(method).expect("invalid method name");
        let sig_name = CString::new(signature).expect("invalid method signature");
        unsafe {
            method_id = (**self.jni)
                .GetStaticMethodID
                .expect("GetStaticMethodID function not found")(
                self.jni,
                class,
                method_name.as_ptr(),
                sig_name.as_ptr(),
            );
        }

        if self.exception_occurred() || method_id.is_null() {
            let message = format!(
                "{} static method with signature {} not found",
                method, signature
            );
            self.diagnose_exception(&message)?;
            Err(crate::err::Error::Jni(message))
        } else {
            Ok(method_id)
        }
    }

    pub fn get_string_utf_chars<'a>(
        &mut self,
        s: crate::jvmti::jstring,
    ) -> (*const ::std::os::raw::c_char, &'a CStr) {
        let utf_chars;
        let cstr;
        unsafe {
            utf_chars = (**self.jni)
                .GetStringUTFChars
                .expect("GetStringUTFChars function not found")(
                self.jni, s, ptr::null_mut()
            );
            cstr = CStr::from_ptr(utf_chars);
        }

        (utf_chars, cstr)
    }

    pub fn release_string_utf_chars(
        &mut self,
        s: crate::jvmti::jstring,
        utf_chars: *const ::std::os::raw::c_char,
    ) {
        unsafe {
            (**self.jni)
                .ReleaseStringUTFChars
                .expect("ReleaseStringUTFChars function not found")(
                self.jni, s, utf_chars
            );
        }
    }
}
