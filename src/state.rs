use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct State {
    /// ID of current node
    pub node_id: String,
    /// ID of all the nodes declared during init
    pub declared_nodes: HashSet<String>,
    /// Message ID tracker
    /// Using this id to update the message id
    pub message_track_id: usize,
    /// List of all the messages we have received.
    pub messages: HashSet<u32>,
    /// Topology provided by the maelstorm server
    pub topology: HashMap<String, HashSet<String>>,
    /// Direct connected nodes
    pub neighborhood: HashSet<String>,
    /// messages known to node
    pub known: HashMap<String, HashSet<u32>>,
}

impl State {
    pub fn get_and_increment(&mut self) -> usize {
        let local_id_copy = self.message_track_id;
        self.message_track_id += 1;
        local_id_copy
    }

    pub fn add_message(&mut self, message: u32) {
        self.messages.insert(message);
    }

    pub fn update_topology(&mut self, topology: &HashMap<String, HashSet<String>>) {
        for (key, values) in topology {
            self.topology
                .entry(key.to_owned())
                .and_modify(|original_values| *original_values = values.clone())
                .or_insert(values.to_owned());
        }
        self.update_neighborhood();
    }

    pub fn update_neighborhood(&mut self) {
        self.neighborhood = self
            .topology
            .get(&self.node_id)
            .unwrap_or(&self.declared_nodes.iter().cloned().collect())
            .iter()
            .cloned()
            .collect();
    }

    pub fn create_known(&mut self, nodes: &HashSet<String>) {
        for node in nodes {
            self.known.insert(node.clone(), HashSet::new());
        }
    }
}
