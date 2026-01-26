#[derive(Clone, Debug)]
pub struct AxElement {
    pub id: String,
}

impl AxElement {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}
