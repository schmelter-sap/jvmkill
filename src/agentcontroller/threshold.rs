pub struct threshold {
    parameters: super::AgentParameters,
    // circular buffer containing the timestamps of up to count_threshold + 1 OOMs
    events: Vec<u64>,
    eventIndex: i32,
}

impl threshold {
    pub fn new(agent_parameters: super::AgentParameters) -> threshold {
        let mut t = threshold {
            parameters: agent_parameters,
            events: Vec::with_capacity(agent_parameters.count_threshold + 1),
            eventIndex: 0,
        };

        //prefill with a safe value
        for i in 0..agent_parameters.count_threshold {
            t.events[i] = 0;
        }

        t
    }
}

impl super::Heuristic for threshold {
    fn onOOM(&mut self) -> bool {
        unimplemented!()
    }
}
