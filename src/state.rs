#[derive(Debug, Default)]
pub struct State {
    local_id: usize,
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
}
