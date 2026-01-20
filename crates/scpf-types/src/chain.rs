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
            Chain::ZkSync => 324,
            Chain::Linea => 59144,
            Chain::Scroll => 534352,
            Chain::Zora => 7777777,
        }
    }

    pub fn api_base_url(&self) -> &'static str {
        match self {
            Chain::Ethereum => "https://api.etherscan.io/v2/api",
            Chain::Bsc => "https://api.bscscan.com/v2/api",
            Chain::Polygon => "https://api.polygonscan.com/v2/api",
            Chain::Arbitrum => "https://api.arbiscan.io/v2/api",
            Chain::Optimism => "https://api-optimistic.etherscan.io/v2/api",
            Chain::Base => "https://api.basescan.org/v2/api",
            Chain::Avalanche => "https://api.snowtrace.io/v2/api",
            Chain::Fantom => "https://api.ftmscan.com/v2/api",
            Chain::ZkSync => "https://block-explorer-api.mainnet.zksync.io/api",
            Chain::Linea => "https://api.lineascan.build/v2/api",
            Chain::Scroll => "https://api.scrollscan.com/v2/api",
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
