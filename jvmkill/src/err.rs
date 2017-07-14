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
use std::fmt::Formatter;
use std::fmt::Result;
use time::ParseError;

#[derive(Debug)]
pub enum Error {
    Jni(String),
    JvmTi(String, ::jvmti::jint),
    Parse(String, ParseError),
    Io(String, ::std::io::Error),
    ActionUnavailableOnThreadExhaustion(String)
}

impl Error {
    #[allow(unused_variables)]
    pub fn rc(&self) -> ::jvmti::jint {
        match *self {
            Error::JvmTi(ref message, ref rc) => *rc,
            _ => 0
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Error::Jni(ref message) => write!(f, "JNI call failed: {}", message),
            Error::JvmTi(ref message, ref rc) => write!(f, "{}: {:?}", message, rc),
            Error::Parse(ref message, ref parse_error) => write!(f, "{}: parse error: {}", message, parse_error),
            Error::Io(ref message, ref io_error) => write!(f, "{}: I/O error: {}", message, io_error),
            Error::ActionUnavailableOnThreadExhaustion(ref message) => write!(f, "cannot {} since the JVM is unable to create a thread", message)
        }
    }
}
