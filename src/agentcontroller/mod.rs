pub struct agentController<'a> {
    jvmti: ::env::JVMTIEnv,
    heuristic: Box<Heuristic + 'a>,
    actions: Vec<Box<Action>>
}

impl<'a> agentController<'a> {
    pub fn new(ti: ::env::JVMTIEnv, options: *mut ::std::os::raw::c_char) -> Result<agentController<'a>, ::jvmti::jint> {
        self::heaphistogram::heapHistogram::new(ti).map(|action| agentController {
            jvmti: ti,
            heuristic: Box::new(threshold::threshold::new(parseParameters(options))),
            actions: vec![Box::new(action)],
        })
    }
}

impl<'a> Action for agentController<'a> {
    fn onOOM(&self, jni_env: ::env::JNIEnv, resourceExhaustionFlags: ::jvmti::jint) {
        for action in &self.actions {
            action.onOOM(jni_env, resourceExhaustionFlags);
        }
    }
}

unsafe impl<'a> Send for agentController<'a> {}
unsafe impl<'a> Sync for agentController<'a> {}

pub trait Action {
    // See https://docs.oracle.com/javase/8/docs/platform/jvmti/jvmti.html#jvmtiResourceExhaustionFlags
    fn onOOM(&self, jni_env: ::env::JNIEnv, resourceExhaustionFlags: ::jvmti::jint);
}

pub trait Heuristic {
    fn onOOM(&mut self) -> bool;
}

mod heaphistogram;
mod threshold;

/**
 * Struct that holds agent configuration
 */
#[derive(Debug, Copy, Clone)]
pub struct AgentParameters {
    time_threshold: usize,
    count_threshold: usize,
    print_heap_histogram: bool,
    heap_histogram_max_entries: usize,
    print_memory_usage: bool
}

fn parseParameters(options: *mut ::std::os::raw::c_char) -> AgentParameters {
    use std::ffi::CStr;

    let mut timeThreshold: usize = 1;
    let mut countThreshold: usize = 0;
    let mut printHeapHistogram: usize = 0;
    let mut heapHistogramMaxEntries: usize = 100;
    let mut printMemoryUsage: usize = 1;

    let cslice;
    unsafe {
        cslice = CStr::from_ptr(options);
    }
    let s: &str = cslice.to_str().unwrap();
    let options = s.split(",").collect::<Vec<_>>();
    for option in &options {
        let tokens = option.splitn(2, "=").collect::<Vec<_>>();
        assert!(tokens.len() == 2);
        let key = tokens[0];
        let value = tokens[1];
        match key {
            "time" => timeThreshold = value.parse().expect("not a number"),
            "count" => countThreshold = value.parse().expect("not a number"),
            "printHeapHistogram" => printHeapHistogram = value.parse().expect("not a number"),
            "heapHistogramMaxEntries" => heapHistogramMaxEntries = value.parse().expect("not a number"),
            "printMemoryUsage" => printMemoryUsage = value.parse().expect("not a number"),
            _ => assert!(false),
        }
    }

    let ap = AgentParameters{
        time_threshold: timeThreshold,
        count_threshold: countThreshold,
        print_heap_histogram: printHeapHistogram != 0,
        heap_histogram_max_entries: heapHistogramMaxEntries,
        print_memory_usage: printMemoryUsage != 0,
    };

    println!("{:?}", ap);
    ap
}