pub struct rom {
    pub data: Vec<u8>,
}
impl rom {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn load(path: &str) -> Self {
        let data = std::fs::read(path).expect("failed to load ROM");
        Self { data }
    }
}
