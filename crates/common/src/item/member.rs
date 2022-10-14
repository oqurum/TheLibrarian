use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MemberSettings {
    pub page_view: Option<PageView>,
}

impl MemberSettings {
    pub fn get_page_view_default(&self) -> PageView {
        match self.page_view {
            Some(v) => v,
            None => PageView::Viewing,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum PageView {
    Viewing,
    Editing,
}

impl PageView {
    pub fn is_viewing(self) -> bool {
        matches!(self, Self::Viewing)
    }

    pub fn is_editing(self) -> bool {
        matches!(self, Self::Editing)
    }
}
