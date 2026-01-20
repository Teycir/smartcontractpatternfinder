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
    ZkSync,
    Linea,
    Scroll,
    Zora,
}

impl Chain {
    pub fn api_base_url(&self) -> &'static str {
        match self {
            Chain::Ethereum => "https://api.etherscan.io/api",
            Chain::Bsc => "https://api.bscscan.com/api",
            Chain::Polygon => "https://api.polygonscan.com/api",
            Chain::Arbitrum => "https://api.arbiscan.io/api",
            Chain::Optimism => "https://api-optimistic.etherscan.io/api",
            Chain::Base => "https://api.basescan.org/api",
            Chain::Avalanche => "https://api.snowtrace.io/api",
            Chain::Fantom => "https://api.ftmscan.com/api",
            Chain::ZkSync => "https://block-explorer-api.mainnet.zksync.io/api",
            Chain::Linea => "https://api.lineascan.build/api",
            Chain::Scroll => "https://api.scrollscan.com/api",
            Chain::Zora => "https://explorer.zora.energy/api",
        }
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
            Chain::ZkSync => "zksync",
            Chain::Linea => "linea",
            Chain::Scroll => "scroll",
            Chain::Zora => "zora",
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
            "zksync" | "zk" => Ok(Chain::ZkSync),
            "linea" => Ok(Chain::Linea),
            "scroll" => Ok(Chain::Scroll),
            "zora" => Ok(Chain::Zora),
            _ => Err(format!("Unsupported chain: {}", s)),
        }
    }
}
