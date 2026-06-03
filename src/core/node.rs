#[derive(Clone, Debug)]
pub struct Node {
    pub id: String,
}

impl Node {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
        }
    }
}
