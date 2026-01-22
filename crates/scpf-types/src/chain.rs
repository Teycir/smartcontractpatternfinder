use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    Ethereum,
    Bsc,
    Polygon,
    Arbitrum,
    Optimism,
    Base,
    Avalanche,
    Fantom,
    Linea,
    Scroll,
}

impl Chain {
    pub fn chain_id(&self) -> u64 {
        match self {
            Chain::Ethereum => 1,
            Chain::Bsc => 56,
            Chain::Polygon => 137,
            Chain::Arbitrum => 42161,
            Chain::Optimism => 10,
            Chain::Base => 8453,
            Chain::Avalanche => 43114,
            Chain::Fantom => 250,
            Chain::Linea => 59144,
            Chain::Scroll => 534352,
        }
    }

    pub fn supports_v2_api(&self) -> bool {
        true
    }

    pub fn requires_api_key(&self) -> bool {
        true
    }

    pub fn api_base_url(&self) -> &'static str {
        // Etherscan V2 Unified API - single endpoint for all supported chains
        "https://api.etherscan.io/v2/api"
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Bsc => "bsc",
            Chain::Polygon => "polygon",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Base => "base",
            Chain::Avalanche => "avalanche",
            Chain::Fantom => "fantom",
            Chain::Linea => "linea",
            Chain::Scroll => "scroll",
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
            "arbitrum" | "arb" => Ok(Chain::Arbitrum),
            "optimism" | "op" => Ok(Chain::Optimism),
            "base" => Ok(Chain::Base),
            "avalanche" | "avax" => Ok(Chain::Avalanche),
            "fantom" | "ftm" => Ok(Chain::Fantom),
            "linea" => Ok(Chain::Linea),
            "scroll" => Ok(Chain::Scroll),
            _ => Err(format!("Unsupported chain: {}", s)),
        }
    }
}
