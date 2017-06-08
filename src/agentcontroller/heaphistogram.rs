pub struct HeapHistogram {
    jvmti: ::env::JvmTiEnv,
}

impl HeapHistogram {
    pub fn new(jvmti: ::env::JvmTiEnv) -> Result<Self, ::jvmti::jint> {
        Ok(Self {
            jvmti: jvmti
        })
    }
}

impl super::Action for HeapHistogram {
    fn on_oom(&self, jni_env: ::env::JniEnv, resourceExhaustionFlags: ::jvmti::jint) {
    }
}
