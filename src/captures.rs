#[derive(Debug, Clone, Default)]
pub struct Captures(pub Vec<Option<String>>);

impl Captures {
    pub fn new(capture_count: usize) -> Self {
        let captures = vec![None; capture_count];
        Self(captures)
    }

    pub fn capture(&mut self, value: &str, index: usize) {
        if !self.0.is_empty() {
            self.0[index - 1] = Some(value.into());
        }
    }

    pub fn get_capture(&self, idx: usize) -> Option<String> {
        return self.0[idx - 1].as_ref().cloned();
    }

    pub fn debug_print(&self) {
        for (i, capture) in self.0.iter().enumerate() {
            match capture {
                Some(val) => {
                    println!("Capture group {}: '{}']", i + 1, val);
                }
                None => {
                    println!("Capture group {}: None", i + 1);
                }
            }
        }
    }
}