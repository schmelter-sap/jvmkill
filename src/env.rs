use ::std::io::Write;
use ::std::ptr;
use ::std::ffi::CString;
use ::std::sync::Mutex;
use ::jvmti::jvmtiEnv;
use ::jvmti::jrawMonitorID;

pub trait JVMTI {
    fn CreateRawMonitor(&mut self, name: String, monitor: &Mutex<RawMonitorID>) -> Result<(), ::jvmti::jint>;
    fn RawMonitorEnter(&mut self, monitor: &Mutex<RawMonitorID>) -> Option<::jvmti::jint>;
    fn RawMonitorExit(&mut self, monitor: &Mutex<RawMonitorID>) -> Option<::jvmti::jint>;
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
            let rawMonitorEexit = (**self.jvmti).RawMonitorExit.unwrap();
            rc = rawMonitorEexit(self.jvmti, *monitor.lock().unwrap().id);
        }
        if rc != ::jvmti::jvmtiError::JVMTI_ERROR_NONE {
            errln!("ERROR: RawMonitorExit failed: {:?}", rc);
            return Some(::jvmti::JNI_ERR);
        }
        None
    }
}
