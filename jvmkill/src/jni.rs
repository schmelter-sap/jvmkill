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

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use mockall::automock;

use crate::bindings::{jclass, jint, jlong, jmethodID, JNI_TRUE, JNIEnv, jobject, jstring, jvalue};

#[cfg_attr(test, automock)]
pub trait JNI {
    fn call_int_method(&self, instance: jobject, method: jmethodID) -> jint;

    fn call_long_method(&self, instance: jobject, method: jmethodID) -> jlong;

    fn call_object_method(&self, instance: jobject, method: jmethodID) -> Option<jobject>;

    fn call_object_method_a(&self, instance: jobject, method: jmethodID, args: &[jvalue]) -> Option<jobject>;

    fn call_static_object_method(&self, class: jclass, method: jmethodID) -> Option<jobject>;

    fn call_static_object_method_a(&self, class: jclass, method: jmethodID, args: &[jvalue]) -> Option<jobject>;

    fn find_class(&self, class: &str) -> Option<jclass>;

    fn get_method(&self, class: jclass, method: &str, signature: &str) -> Option<jmethodID>;

    fn get_static_method(&self, class: jclass, method: &str, signature: &str) -> Option<jmethodID>;

    fn get_string_utf_chars(&self, s: jstring) -> Option<String>;

    fn new_string_utf(&self, s: &str) -> jstring;
}

pub struct DefaultJNI {
    internal: *mut JNIEnv
}

impl DefaultJNI {
    pub fn new(jni_env: *mut JNIEnv) -> Self {
        return Self { internal: jni_env };
    }

    fn exception_check(&self) -> bool {
        let f = unsafe { (**self.internal).ExceptionCheck }
            .expect("JNIEnv.ExceptionCheck not found");

        return unsafe { f(self.internal) == JNI_TRUE as u8 };
    }

    fn exception_describe(&self) {
        let f = unsafe { (**self.internal).ExceptionDescribe }
            .expect("JNIEnv.ExceptionDescribe not found");

        unsafe { f(self.internal) };
    }

    fn release_string_utf_chars(&self, s: jstring, p: *const c_char) {
        let f = unsafe { (**self.internal).ReleaseStringUTFChars }
            .expect("JNIEnv.ReleaseStringUTFChars not found");

        unsafe { f(self.internal, s, p) };
    }
}

impl JNI for DefaultJNI {
    fn call_int_method(&self, instance: jobject, method: jmethodID) -> jint {
        let f = unsafe { (**self.internal).CallIntMethod }
            .expect("JNIEnv.CallIntMethod not found");

        let r = unsafe { f(self.internal, instance, method) };
        if self.exception_check() {
            self.exception_describe();
            panic!();
        } else {
            return r;
        }
    }

    fn call_long_method(&self, instance: jobject, method: jmethodID) -> jlong {
        let f = unsafe { (**self.internal).CallLongMethod }
            .expect("JNIEnv.CallLongMethod not found");

        let r = unsafe { f(self.internal, instance, method) };
        if self.exception_check() {
            self.exception_describe();
            panic!();
        } else {
            return r;
        }
    }

    fn call_object_method(&self, instance: jobject, method: jmethodID) -> Option<jobject> {
        let f = unsafe { (**self.internal).CallObjectMethod }
            .expect("JNIEnv.CallObjectMethod not found");

        let r = unsafe { f(self.internal, instance, method) };
        if self.exception_check() {
            self.exception_describe();
            panic!();
        } else if r == ptr::null_mut() {
            return None;
        } else {
            return Some(r);
        }
    }

    fn call_object_method_a(&self, instance: jobject, method: jmethodID, args: &[jvalue]) -> Option<jobject> {
        let f = unsafe { (**self.internal).CallObjectMethodA }
            .expect("JNIEnv.CallObjectMethodA not found");

        let r = unsafe { f(self.internal, instance, method, &args[0]) };
        if self.exception_check() {
            self.exception_describe();
            panic!();
        } else if r == ptr::null_mut() {
            return None;
        } else {
            return Some(r);
        }
    }

    fn call_static_object_method(&self, class: jclass, method: jmethodID) -> Option<jobject> {
        let f = unsafe { (**self.internal).CallStaticObjectMethod }
            .expect("JNIEnv.CallStaticObjectMethod not found");

        let r = unsafe { f(self.internal, class, method) };
        if self.exception_check() {
            self.exception_describe();
            panic!();
        } else if r == ptr::null_mut() {
            return None;
        } else {
            return Some(r);
        }
    }

    fn call_static_object_method_a(&self, class: jclass, method: jmethodID, args: &[jvalue]) -> Option<jobject> {
        let f = unsafe { (**self.internal).CallStaticObjectMethodA }
            .expect("JNIEnv.CallStaticObjectMethodA not found");

        let r = unsafe { f(self.internal, class, method, &args[0]) };
        if self.exception_check() {
            self.exception_describe();
            panic!();
        } else if r == ptr::null_mut() {
            return None;
        } else {
            return Some(r);
        }
    }

    fn find_class(&self, class: &str) -> Option<jclass> {
        let c = CString::new(class)
            .expect("unable to create CString");

        let f = unsafe { (**self.internal).FindClass }
            .expect("JNIEnv.FindClass not found");

        let r = unsafe { f(self.internal, c.as_ptr()) };
        if self.exception_check() {
            self.exception_describe();
            panic!();
        } else if r == ptr::null_mut() {
            return None;
        } else {
            return Some(r);
        }
    }

    fn get_method(&self, class: jclass, method: &str, signature: &str) -> Option<jmethodID> {
        let m = CString::new(method)
            .expect("unable to create CString");
        let s = CString::new(signature)
            .expect("unable to create CString");

        let f = unsafe { (**self.internal).GetMethodID }
            .expect("JNIEnv.GetMethodID not found");

        let r = unsafe { f(self.internal, class, m.as_ptr(), s.as_ptr()) };
        if r == ptr::null_mut() {
            return None;
        } else {
            return Some(r);
        }
    }

    fn get_static_method(&self, class: jclass, method: &str, signature: &str) -> Option<jmethodID> {
        let m = CString::new(method)
            .expect("unable to create CString");
        let s = CString::new(signature)
            .expect("unable to create CString");

        let f = unsafe { (**self.internal).GetStaticMethodID }
            .expect("JNIEnv.GetStaticMethodID not found");

        let r = unsafe { f(self.internal, class, m.as_ptr(), s.as_ptr()) };
        if r == ptr::null_mut() {
            return None;
        } else {
            return Some(r);
        }
    }

    fn get_string_utf_chars(&self, s: jstring) -> Option<String> {
        let f = unsafe { (**self.internal).GetStringUTFChars }
            .expect("JNIEnv.GetStringUTFChars not found");

        let r = unsafe { f(self.internal, s, ptr::null_mut()) };
        if r == ptr::null_mut() {
            return None;
        }

        let c = String::from(unsafe { CStr::from_ptr(r) }
            .to_string_lossy());

        self.release_string_utf_chars(s, r);

        return Some(c);
    }

    fn new_string_utf(&self, s: &str) -> jstring {
        let c = CString::new(s)
            .expect("unable to create CString");

        let f = unsafe { (**self.internal).NewStringUTF }
            .expect("JNIEnv.NewStringUTF not found");

        return unsafe { f(self.internal, c.as_ptr()) };
    }
}
