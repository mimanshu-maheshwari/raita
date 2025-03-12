use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct State {
    pub node_id: String,
    pub declared_nodes: HashSet<String>,
    pub local_id: usize,
    pub messages: Vec<u32>,
    pub topology: HashMap<String, HashSet<String>>,
}

impl State {
    #[inline(always)]
    pub fn get_and_increment(&mut self) -> usize {
        let local_id_copy = self.local_id;
        self.local_id += 1;
        local_id_copy
    }

    #[inline(always)]
    pub fn add_message(&mut self, message: u32) {
        self.messages.push(message);
    }

    #[inline(always)]
    pub fn update_topology(&mut self, topology: &HashMap<String, HashSet<String>>) {
        for (key, values) in topology {
            self.topology
                .entry(key.to_owned())
                .and_modify(|original_values| *original_values = values.clone())
                .or_insert(values.to_owned());
        }
    }
}
