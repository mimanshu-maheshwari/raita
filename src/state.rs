use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct State {
    local_id: usize,
    messages: Vec<u32>,
    topology: HashMap<String, HashSet<String>>,
}

impl State {
    pub fn local_id(&self) -> usize {
        self.local_id
    }

    pub fn set_local_id(&mut self, local_id: usize) {
        self.local_id = local_id;
    }
    pub fn get_and_increment(&mut self) -> usize {
        let local_id_copy = self.local_id;
        self.local_id += 1;
        local_id_copy
    }

    pub fn messages(&self) -> &[u32] {
        &self.messages
    }

    pub fn topology(&self) -> &HashMap<String, HashSet<String>> {
        &self.topology
    }

    pub fn add_message(&mut self, message: u32) {
        self.messages.push(message);
    }

    pub fn update_topology(&mut self, topology: &HashMap<String, HashSet<String>>) {
        for (key, values) in topology {
            self.topology
                .entry(key.to_owned())
                .and_modify(|original_values| *original_values = values.clone())
                .or_insert(values.to_owned());
        }
    }
}
