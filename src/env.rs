use ::std::io::Write;
use std::mem::size_of;
use ::std::ptr;
use ::std::ffi::CString;
use ::std::sync::Mutex;
use ::jvmti::jvmtiEnv;
use ::jvmti::jrawMonitorID;

pub trait JvmTI {
    fn create_raw_monitor(&mut self, name: String, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint>;
    fn raw_monitor_enter(&mut self, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint>;
    fn raw_monitor_exit(&mut self, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint>;
    fn on_resource_exhausted(&mut self, callback: FnResourceExhausted) -> Result<(), ::jvmti::jint>;
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
pub struct JvmTIEnv {
    jvmti: *mut jvmtiEnv
}

impl JvmTIEnv {
    pub fn new(vm: *mut ::jvmti::JavaVM) -> Result<JvmTIEnv, ::jvmti::jint> {
        let mut penv: *mut ::std::os::raw::c_void = ptr::null_mut();
        let rc;
        unsafe {
            rc = (**vm).GetEnv.unwrap()(vm, &mut penv, ::jvmti::JVMTI_VERSION as i32);
        }
        if rc as u32 != ::jvmti::JNI_OK {
            eprintln!("ERROR: GetEnv failed: {}", rc);
            return Err(::jvmti::JNI_ERR);
        }
        Ok(JvmTIEnv { jvmti: penv as *mut jvmtiEnv })
    }

    pub fn wrap(jvmti_env: *mut jvmtiEnv) -> JvmTIEnv {
        JvmTIEnv { jvmti: jvmti_env }
    }
}

impl JvmTI for JvmTIEnv {
    fn create_raw_monitor(&mut self, name: String, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint> {
        let rc;
        unsafe {
            let create_raw_monitor_fn = (**self.jvmti).CreateRawMonitor.unwrap();
            rc = create_raw_monitor_fn(self.jvmti, CString::new(name).unwrap().into_raw(), monitor.lock().unwrap().id);
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            eprintln!("ERROR: CreateRawMonitor failed: {:?}", rc);
            return Err(::jvmti::JNI_ERR);
        }
        Ok(())
    }

    fn raw_monitor_enter(&mut self, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint> {
        let rc;
        unsafe {
            let raw_monitor_enter_fn = (**self.jvmti).RawMonitorEnter.unwrap();
            rc = raw_monitor_enter_fn(self.jvmti, *monitor.lock().unwrap().id);
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            eprintln!("ERROR: RawMonitorEnter failed: {:?}", rc);
            return Err(::jvmti::JNI_ERR);
        }
        Ok(())
    }

    fn raw_monitor_exit(&mut self, monitor: &Mutex<RawMonitorId>) -> Result<(), ::jvmti::jint> {
        let rc;
        unsafe {
            let raw_monitor_exit_fn = (**self.jvmti).RawMonitorExit.unwrap();
            rc = raw_monitor_exit_fn(self.jvmti, *monitor.lock().unwrap().id);
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            eprintln!("ERROR: RawMonitorExit failed: {:?}", rc);
            return Err(::jvmti::JNI_ERR);
        }
        Ok(())
    }

    fn on_resource_exhausted(&mut self, callback: FnResourceExhausted) -> Result<(), ::jvmti::jint> {
        unsafe {
            EVENT_CALLBACKS.resource_exhausted = Some(callback);
        }

        let rc;
        unsafe {
            let set_event_callbacks_fn = (**self.jvmti).SetEventCallbacks.unwrap();
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
            rc = set_event_callbacks_fn(self.jvmti, &callbacks, size_of::<::jvmti::jvmtiEventCallbacks>() as i32);
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            eprintln!("ERROR: SetEventCallbacks failed: {:?}", rc);
            return Err(::jvmti::JNI_ERR);
        }

        let rc;
        unsafe {
            let set_event_notification_mode_fn = (**self.jvmti).SetEventNotificationMode.unwrap();
            rc = set_event_notification_mode_fn(self.jvmti, ::jvmti::jvmtiEventMode::JVMTI_ENABLE, ::jvmti::jvmtiEvent::JVMTI_EVENT_RESOURCE_EXHAUSTED, ::std::ptr::null_mut());
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            eprintln!("ERROR: SetEventNotificationMode failed: {:?}", rc);
            return Err(::jvmti::JNI_ERR);
        }

        Ok(())
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
            let jvmti_env = JvmTIEnv::wrap(jvmti_env);
            function(jvmti_env, JniEnv::new(jni_env), flags);
        }
        None => println!("No resource exhaustion exit registered")
    }
}

pub type FnResourceExhausted = fn(jvmti_env: JvmTIEnv, jni_env: JniEnv, flags: ::jvmti::jint);

#[derive(Default, Clone)]
pub struct EventCallbacks {
    pub resource_exhausted: Option<FnResourceExhausted>
}

pub static mut EVENT_CALLBACKS: EventCallbacks = EventCallbacks {
    resource_exhausted: None
};

#[derive(Clone, Copy)]
pub struct JniEnv {
    jni: *mut ::jvmti::JNIEnv
}

impl JniEnv {
    pub fn new(jni_env: *mut ::jvmti::JNIEnv) -> JniEnv {
           JniEnv {jni: jni_env}
    }
}
