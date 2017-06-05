use ::std::io::Write;
use std::mem::size_of;
use ::std::ptr;
use ::std::ffi::CString;
use ::std::sync::Mutex;
use ::jvmti::jvmtiEnv;
use ::jvmti::jrawMonitorID;

pub trait JVMTI {
    fn CreateRawMonitor(&mut self, name: String, monitor: &Mutex<RawMonitorID>) -> Result<(), ::jvmti::jint>;
    fn RawMonitorEnter(&mut self, monitor: &Mutex<RawMonitorID>) -> Option<::jvmti::jint>;
    fn RawMonitorExit(&mut self, monitor: &Mutex<RawMonitorID>) -> Option<::jvmti::jint>;
    fn OnResourceExhausted(&mut self, callback: FnResourceExhausted) -> Option<::jvmti::jint>;
}

pub struct RawMonitorID {
    id: *mut jrawMonitorID,
}

impl RawMonitorID {
    pub fn new() -> RawMonitorID {
        RawMonitorID {
            id: Box::into_raw(Box::new(ptr::null_mut())),
        }
    }
}

unsafe impl Send for RawMonitorID {}

#[derive(Clone, Copy)]
pub struct JVMTIEnv {
    jvmti: *mut jvmtiEnv
}

macro_rules! errln (
    ($($arg:tt)*) => { {
        writeln!(&mut ::std::io::stderr(), $($arg)*).unwrap();
    } }
);

impl JVMTIEnv {
    pub fn new(vm: *mut ::jvmti::JavaVM) -> Result<JVMTIEnv, ::jvmti::jint> {
        let mut penv: *mut ::std::os::raw::c_void = ptr::null_mut();
        let rc;
        unsafe {
            rc = (**vm).GetEnv.unwrap()(vm, &mut penv, ::jvmti::JVMTI_VERSION as i32);
        }
        if rc as u32 != ::jvmti::JNI_OK {
            errln!("ERROR: GetEnv failed: {}", rc);
            return Err(::jvmti::JNI_ERR);
        }
        Ok(JVMTIEnv { jvmti: penv as *mut jvmtiEnv })
    }

    pub fn wrap(jvmti_env: *mut jvmtiEnv) -> JVMTIEnv {
        JVMTIEnv { jvmti: jvmti_env }
    }
}

impl JVMTI for JVMTIEnv {
    fn CreateRawMonitor(&mut self, name: String, monitor: &Mutex<RawMonitorID>) -> Result<(), ::jvmti::jint> {
        let rc;
        unsafe {
            let createRawMonitor = (**self.jvmti).CreateRawMonitor.unwrap();
            rc = createRawMonitor(self.jvmti, CString::new(name).unwrap().into_raw(), monitor.lock().unwrap().id);
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            errln!("ERROR: CreateRawMonitor failed: {:?}", rc);
            return Err(::jvmti::JNI_ERR);
        }
        Ok(())
    }

    fn RawMonitorEnter(&mut self, monitor: &Mutex<RawMonitorID>) -> Option<::jvmti::jint> {
        let rc;
        unsafe {
            let rawMonitorEnter = (**self.jvmti).RawMonitorEnter.unwrap();
            rc = rawMonitorEnter(self.jvmti, *monitor.lock().unwrap().id);
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            errln!("ERROR: RawMonitorEnter failed: {:?}", rc);
            return Some(::jvmti::JNI_ERR);
        }
        None
    }

    fn RawMonitorExit(&mut self, monitor: &Mutex<RawMonitorID>) -> Option<::jvmti::jint> {
        let rc;
        unsafe {
            let rawMonitorExit = (**self.jvmti).RawMonitorExit.unwrap();
            rc = rawMonitorExit(self.jvmti, *monitor.lock().unwrap().id);
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            errln!("ERROR: RawMonitorExit failed: {:?}", rc);
            return Some(::jvmti::JNI_ERR);
        }
        None
    }

    fn OnResourceExhausted(&mut self, callback: FnResourceExhausted) -> Option<::jvmti::jint> {
        unsafe {
            EVENT_CALLBACKS.resource_exhausted = Some(callback);
        }

        let rc;
        unsafe {
            let setEventCallbacks = (**self.jvmti).SetEventCallbacks.unwrap();
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
            rc = setEventCallbacks(self.jvmti, &callbacks, size_of::<::jvmti::jvmtiEventCallbacks>() as i32);
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            errln!("ERROR: RawMonitorExit failed: {:?}", rc);
            return Some(::jvmti::JNI_ERR);
        }

        let rc;
        unsafe {
            let setEventNotificationMode = (**self.jvmti).SetEventNotificationMode.unwrap();
            rc = setEventNotificationMode(self.jvmti, ::jvmti::jvmtiEventMode::JVMTI_ENABLE, ::jvmti::jvmtiEvent::JVMTI_EVENT_RESOURCE_EXHAUSTED, ::std::ptr::null_mut());
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            errln!("ERROR: RawMonitorExit failed: {:?}", rc);
            return Some(::jvmti::JNI_ERR);
        }

        None
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
            let jvmti_env = JVMTIEnv::wrap(jvmti_env);
            function(jvmti_env, flags);
        }
        None => println!("No resource exhaustion exit registered")
    }
}

pub type FnResourceExhausted = fn(jvmti_env: JVMTIEnv, flags: ::jvmti::jint);

#[derive(Default, Clone)]
pub struct EventCallbacks {
    pub resource_exhausted: Option<FnResourceExhausted>
}

pub static mut EVENT_CALLBACKS: EventCallbacks = EventCallbacks {
    resource_exhausted: None
};
