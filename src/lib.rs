#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::sync::Mutex;
use std::io::Write;

#[macro_use]
mod env;
mod jvmti;
mod agentcontroller;

use env::JvmTI;
use agentcontroller::Action;

#[macro_use]
extern crate lazy_static;
extern crate libc;

lazy_static! {
    static ref STATIC_CONTEXT: Mutex<AgentContext<'static>> = Mutex::new(AgentContext::new());
    static ref RAW_MONITOR_ID: Mutex<env::RawMonitorId> = Mutex::new(env::RawMonitorId::new());
}

#[derive(Default)]
pub struct AgentContext<'a> {
    ac: Option<agentcontroller::controller::AgentController<'a>>
}

impl<'a> AgentContext<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set(&mut self, a: agentcontroller::controller::AgentController<'a>) {
        self.ac = Some(a);
    }

    pub fn on_oom(&self, jni_env: ::env::JniEnv, resourceExhaustionFlags: ::jvmti::jint) {
        self.ac.as_ref().map(|a| a.on_oom(jni_env, resourceExhaustionFlags));
    }
}

#[no_mangle]
#[allow(unused_variables)]
pub extern fn Agent_OnLoad(vm: *mut jvmti::JavaVM, options: *mut ::std::os::raw::c_char,
                           reserved: *mut ::std::os::raw::c_void) -> jvmti::jint {
    let jvmti_env = env::JvmTIEnv::new(vm);

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

fn resource_exhausted(mut jvmti_env: env::JvmTIEnv, jni_env: env::JniEnv, flags: ::jvmti::jint) {
    println!("Resource exhausted callback driven!");

    if let Err(err) = jvmti_env.raw_monitor_enter(&RAW_MONITOR_ID) {
        eprintln!("ERROR: RawMonitorEnter failed: {}", err);
        return
    }

    STATIC_CONTEXT.lock().map(|a| a.on_oom(jni_env, flags)).unwrap();

    if let Err(err) = jvmti_env.raw_monitor_exit(&RAW_MONITOR_ID) {
        eprintln!("ERROR: RawMonitorExit failed: {}", err);
        return
    }
}
