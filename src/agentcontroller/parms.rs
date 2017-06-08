/**
 * Struct that holds agent configuration
 */
#[derive(Debug, Copy, Clone)]
pub struct AgentParameters {
    pub time_threshold: usize,
    pub count_threshold: usize,
    pub print_heap_histogram: bool,
    pub heap_histogram_max_entries: usize,
    pub print_memory_usage: bool
}

impl AgentParameters {
    pub fn parseParameters(options: *mut ::std::os::raw::c_char) -> AgentParameters {
        use std::ffi::CStr;

        let mut time_threshold: usize = 1;
        let mut count_threshold: usize = 0;
        let mut print_heap_histogram: usize = 0;
        let mut heap_histogram_max_entries: usize = 100;
        let mut print_memory_usage: usize = 1;

        let cslice;
        unsafe {
            cslice = CStr::from_ptr(options);
        }
        let s: &str = cslice.to_str().unwrap();
        let options = s.split(',').collect::<Vec<_>>();
        for option in &options {
            let tokens = option.splitn(2, '=').collect::<Vec<_>>();
            assert_eq!(tokens.len(), 2);
            let key = tokens[0];
            let value = tokens[1];
            match key {
                "time" => time_threshold = value.parse().expect("not a number"),
                "count" => count_threshold = value.parse().expect("not a number"),
                "printHeapHistogram" => print_heap_histogram = value.parse().expect("not a number"),
                "heapHistogramMaxEntries" => heap_histogram_max_entries = value.parse().expect("not a number"),
                "printMemoryUsage" => print_memory_usage = value.parse().expect("not a number"),
                _ => assert!(false),
            }
        }

        let ap = AgentParameters {
            time_threshold: time_threshold,
            count_threshold: count_threshold,
            print_heap_histogram: print_heap_histogram != 0,
            heap_histogram_max_entries: heap_histogram_max_entries,
            print_memory_usage: print_memory_usage != 0,
        };

        println!("AgentParameters={:?}", ap);
        ap
    }
}
