pub struct HeapHistogram {
    jvmti: ::env::JvmTIEnv,
}

impl HeapHistogram {
    pub fn new(jvmti: ::env::JvmTIEnv) -> Result<HeapHistogram, ::jvmti::jint> {
        Ok(HeapHistogram {
            jvmti: jvmti
        })
    }
}

impl super::Action for HeapHistogram {
    fn on_oom(&self, jni_env: ::env::JniEnv, resourceExhaustionFlags: ::jvmti::jint) {
        println!("in heapHistogram.on_oom");
    }
}
