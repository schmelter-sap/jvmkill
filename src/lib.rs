#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::sync::Mutex;
use std::io::Write;

#[macro_use]
mod env;
mod jvmti;
mod agentcontroller;

use env::JVMTI;
use agentcontroller::Action;

#[macro_use]
extern crate lazy_static;
extern crate libc;

lazy_static! {
    static ref STATIC_CONTEXT: Mutex<AgentContext> = Mutex::new(AgentContext::new());
    static ref RAW_MONITOR_ID: Mutex<env::RawMonitorID> = Mutex::new(env::RawMonitorID::new());
}

pub struct AgentContext {
    ac: Option<agentcontroller::agentController>
}

impl AgentContext {
    pub fn new() -> AgentContext {
        AgentContext {
            ac: None,
        }
    }

    pub fn set(&mut self, a: agentcontroller::agentController) {
        self.ac = Some(a);
    }

    pub fn onOOM(&self, jni_env: ::env::JNIEnv, resourceExhaustionFlags: ::jvmti::jint) {
        self.ac.as_ref().map(|a| a.onOOM(jni_env, resourceExhaustionFlags));
    }
}

#[no_mangle]
#[allow(unused_variables)]
pub extern fn Agent_OnLoad(vm: *mut jvmti::JavaVM, options: *mut ::std::os::raw::c_char,
                           reserved: *mut ::std::os::raw::c_void) -> jvmti::jint {
    let jvmti_env = env::JVMTIEnv::new(vm);

    if let Err(e) = jvmti_env
        .and_then(|ti| agentcontroller::agentController::new(ti))
        .map(|ac| STATIC_CONTEXT.lock().unwrap().set(ac)) {
        return e;
    }

    if let Err(e) = jvmti_env
        .and_then(|mut ti| {
            ti.OnResourceExhausted(resourceExhausted);
            ti.CreateRawMonitor(String::from("jvmkill"), &RAW_MONITOR_ID)
        }) {
        return e;
    }

    0
}

fn resourceExhausted(mut jvmti_env: env::JVMTIEnv, jni_env: env::JNIEnv, flags: ::jvmti::jint) {
    println!("Resource exhausted callback driven!");

    if let Err(err) = jvmti_env.RawMonitorEnter(&RAW_MONITOR_ID) {
        eprintln!("ERROR: RawMonitorEnter failed: {}", err);
        return
    }

    STATIC_CONTEXT.lock().map(|a| a.onOOM(jni_env, flags));

    if let Err(err) = jvmti_env.RawMonitorExit(&RAW_MONITOR_ID) {
        eprintln!("ERROR: RawMonitorExit failed: {}", err);
        return
    }
}
