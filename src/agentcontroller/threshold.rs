pub struct Threshold {
    parameters: super::parms::AgentParameters,
    // circular buffer containing the timestamps of up to count_threshold + 1 OOMs
    events: Vec<u64>,
    event_index: i32,
}

impl Threshold {
    pub fn new(agent_parameters: super::parms::AgentParameters) -> Threshold {
        let mut t = Threshold {
            parameters: agent_parameters,
            events: Vec::with_capacity(agent_parameters.count_threshold + 1),
            event_index: 0,
        };

        //prefill with a safe value
        for i in 0..agent_parameters.count_threshold {
            t.events[i] = 0;
        }

        t
    }
}

impl super::Heuristic for Threshold {
    fn on_oom(&mut self) -> bool {
        unimplemented!()
    }
}
