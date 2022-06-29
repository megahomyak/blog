use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq)]
pub struct PageColors {
    title: String,
    background: String,
}

impl PageColors {
    pub fn new<T: Into<String>, B: Into<String>>(title: T, background: B) -> Self {
        Self { background: background.into(), title: title.into() }
    }

    pub fn background(&self) -> &str {
        self.background.as_ref()
    }

    pub fn title(&self) -> &str {
        self.title.as_ref()
    }
}
