use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    Ethereum,
    Bsc,
    Polygon,
}

impl Chain {
    pub fn api_base_url(&self) -> &'static str {
        match self {
            Chain::Ethereum => "https://api.etherscan.io/api",
            Chain::Bsc => "https://api.bscscan.com/api",
            Chain::Polygon => "https://api.polygonscan.com/api",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Bsc => "bsc",
            Chain::Polygon => "polygon",
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Chain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" => Ok(Chain::Ethereum),
            "bsc" | "binance" => Ok(Chain::Bsc),
            "polygon" | "matic" => Ok(Chain::Polygon),
            _ => Err(format!("Unsupported chain: {}", s)),
        }
    }
}
