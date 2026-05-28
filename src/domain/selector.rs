#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorType {
    Css,
    XPath,
}

impl SelectorType {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "css" => Some(Self::Css),
            "xpath" => Some(Self::XPath),
            _ => None,
        }
    }
}
