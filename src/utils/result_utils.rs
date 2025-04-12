pub struct CollectionResult<T> {
    pub result: Vec<T>,
    pub errors: Vec<eyre::Report>,
}

impl<T> CollectionResult<T> {
    pub fn new() -> Self {
        Self { result: Vec::new(), errors: Vec::new() }
    }
}
