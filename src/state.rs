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

    pub fn add_messages<'a>(&mut self, messages: impl IntoIterator<Item = &'a u32>) {
        self.messages.extend(messages.into_iter().copied());
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
        let neighborhood = self
            .topology
            .get(&self.node_id)
            .cloned()
            .unwrap_or_else(|| self.declared_nodes.clone());

        self.neighborhood = neighborhood
            .into_iter()
            .filter(|node| node != &self.node_id)
            .collect();

        for node in self.neighborhood.iter() {
            self.known.entry(node.clone()).or_default();
        }
    }

    pub fn create_known(&mut self, nodes: &HashSet<String>) {
        for node in nodes {
            if node != &self.node_id {
                self.known.entry(node.clone()).or_default();
            }
        }
    }

    pub fn messages_unknown_to(&self, node: &str) -> HashSet<u32> {
        let known = self.known.get(node);
        self.messages
            .iter()
            .copied()
            .filter(|message| !known.is_some_and(|known| known.contains(message)))
            .collect()
    }

    pub fn mark_known<'a>(&mut self, node: &str, messages: impl IntoIterator<Item = &'a u32>) {
        if node == self.node_id {
            return;
        }

        self.known
            .entry(node.to_owned())
            .or_default()
            .extend(messages.into_iter().copied());
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn neighborhood_excludes_current_node() {
        let mut state = State {
            node_id: "n1".to_owned(),
            declared_nodes: HashSet::from([
                "n0".to_owned(),
                "n1".to_owned(),
                "n2".to_owned(),
            ]),
            ..State::default()
        };

        state.update_neighborhood();

        assert_eq!(
            state.neighborhood,
            HashSet::from(["n0".to_owned(), "n2".to_owned()])
        );
    }

    #[test]
    fn topology_refreshes_neighborhood_and_known_peers() {
        let mut state = State {
            node_id: "n1".to_owned(),
            ..State::default()
        };

        state.update_topology(&HashMap::from([(
            "n1".to_owned(),
            HashSet::from(["n1".to_owned(), "n3".to_owned()]),
        )]));

        assert_eq!(state.neighborhood, HashSet::from(["n3".to_owned()]));
        assert!(state.known.contains_key("n3"));
        assert!(!state.known.contains_key("n1"));
    }

    #[test]
    fn unknown_messages_only_returns_messages_missing_from_peer() {
        let mut state = State {
            node_id: "n0".to_owned(),
            messages: HashSet::from([10, 11, 12]),
            ..State::default()
        };
        state.mark_known("n1", [10, 12].iter());

        assert_eq!(state.messages_unknown_to("n1"), HashSet::from([11]));
        assert_eq!(state.messages_unknown_to("n2"), HashSet::from([10, 11, 12]));
    }
}
