#[derive(Debug, Default)]
pub struct CollectionResult<T> {
    pub results: Vec<T>,
    pub errors: Vec<eyre::Report>,
}

impl<T> CollectionResult<T> {
    pub fn new() -> Self {
        Self { results: Vec::new(), errors: Vec::new() }
    }
}
