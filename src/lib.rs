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
        match self.ac {
            Some(_) => panic!("Agent controller has already been set"),
            None => self.ac = Some(a),
        }
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
    let ac = jvmti_env.and_then(|ti| agentcontroller::agentController::new(ti));

    match ac {
        Ok(a) =>
            STATIC_CONTEXT.lock().unwrap().set(a),
        Err(e) => {
            return 1;
        }
    }

    let rc: Result<(), jvmti::jint> = jvmti_env.and_then(|mut ti| {
        ti.OnResourceExhausted(resourceExhausted);

        ti.CreateRawMonitor(String::from("jvmkill"), &RAW_MONITOR_ID)
    });

    match rc {
        Ok(_) => 0,
        Err(e) => e
    }
}

fn resourceExhausted(mut jvmti_env: env::JVMTIEnv, jni_env: env::JNIEnv, flags: ::jvmti::jint) {
    println!("Resource exhausted callback driven!");

    let rc = jvmti_env.RawMonitorEnter(&RAW_MONITOR_ID);
    if let Err(err) = rc {
        eprintln!("ERROR: RawMonitorEnter failed: {}", err);
        return
    }

    STATIC_CONTEXT.lock().map(|a| a.onOOM(jni_env, flags));
    
    let rc = jvmti_env.RawMonitorExit(&RAW_MONITOR_ID);
    if let Err(err) = rc {
        eprintln!("ERROR: RawMonitorExit failed: {}", err);
        return
    }
}
