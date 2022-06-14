use std::borrow::Cow;

use serde::{Deserialize, Serialize};

pub type CowStr = Cow<'static, str>;
pub type CssColor = CowStr;

#[derive(Deserialize, Serialize)]
pub struct PageColors {
    background: CssColor,
    title: CssColor,
}

impl PageColors {
    pub fn new<Bg, Title>(background: Bg, title: Title) -> Self
    where
        Bg: Into<CowStr>,
        Title: Into<CowStr>,
    {
        Self {
            background: background.into(),
            title: title.into(),
        }
    }

    pub const fn title(&self) -> &Cow<str> {
        &self.title
    }

    pub const fn background(&self) -> &Cow<str> {
        &self.background
    }
}
