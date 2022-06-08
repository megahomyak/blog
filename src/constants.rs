pub const DATE_FORMAT: &str = "%Y.%m.%d";

pub struct PageColorCodes {
    title: &'static str,
    background: &'static str,
}
impl PageColorCodes {
    pub const fn title(&self) -> &'static str {
        self.title
    }

    pub const fn background(&self) -> &'static str {
        self.background
    }
}
pub const PAGE_COLORS: [PageColorCodes; 6] = [
    PageColorCodes {
        title: "C8566B",
        background: "F6E5E8",
    },
    PageColorCodes {
        title: "E78963",
        background: "FBEDE7",
    },
    PageColorCodes {
        title: "F2D48F",
        background: "FDF8EE",
    },
    PageColorCodes {
        title: "9D75BF",
        background: "F0EAF5",
    },
    PageColorCodes {
        title: "9EC299",
        background: "F0F5EF",
    },
    PageColorCodes {
        title: "6661AB",
        background: "E8E7F2",
    },
];

pub const ARTICLES_DIRECTORY_NAME: &str = "articles";
pub const FILES_DIRECTORY_NAME: &str = "files";

pub const AUTHOR_NAME: &str = "megahomyak";
