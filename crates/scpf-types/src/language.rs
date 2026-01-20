#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Solidity,
    Vyper,
}

impl Language {
    pub fn detect(source: &str) -> Self {
        if source.contains("@version")
            || source.contains("@external")
            || source.contains("@internal")
            || source.contains("def ")
            || source.contains("event ") && source.contains(":")
            || source.contains("struct ") && source.contains(":")
        {
            Language::Vyper
        } else {
            Language::Solidity
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            Language::Solidity => ".sol",
            Language::Vyper => ".vy",
        }
    }
}
