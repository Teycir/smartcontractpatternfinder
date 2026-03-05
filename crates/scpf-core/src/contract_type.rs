#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractType {
    Token,
    NFT,
    DEX,
    Lending,
    Staking,
    Governance,
    Unknown,
}

pub struct ContractTypeDetector;

impl ContractTypeDetector {
    pub fn detect(source: &str) -> ContractType {
        // Quick heuristic-based detection
        let lower = source.to_lowercase();

        // NFT detection
        if lower.contains("erc721") || lower.contains("erc1155") {
            return ContractType::NFT;
        }

        // DEX detection
        if (lower.contains("uniswap") || lower.contains("swap") || lower.contains("pool"))
            && (lower.contains("sqrtpricex96")
                || lower.contains("slot0")
                || lower.contains("liquidity"))
        {
            return ContractType::DEX;
        }

        // Token detection
        if lower.contains("erc20") || (lower.contains("transfer") && lower.contains("balanceof")) {
            return ContractType::Token;
        }

        // Lending detection
        if lower.contains("borrow") || lower.contains("collateral") || lower.contains("liquidate") {
            return ContractType::Lending;
        }

        // Staking detection
        if lower.contains("stake") || lower.contains("reward") {
            return ContractType::Staking;
        }

        // Governance detection
        if lower.contains("propose") || lower.contains("vote") || lower.contains("timelock") {
            return ContractType::Governance;
        }

        ContractType::Unknown
    }

    pub fn should_skip_pattern(contract_type: ContractType, pattern_id: &str) -> bool {
        match (contract_type, pattern_id) {
            // Skip DEX patterns for NFT contracts
            (ContractType::NFT, "sqrt-price-no-bounds") => true,
            (ContractType::NFT, "uniswap-getamountout-no-twap") => true,
            (ContractType::NFT, "price-from-single-pool") => true,

            // Skip NFT patterns for DEX contracts
            (ContractType::DEX, "external-mint-no-modifier") => true,
            (ContractType::DEX, "external-burn-no-modifier") => true,

            // Skip token patterns for NFT contracts
            (ContractType::NFT, "spot-price-from-reserves") => true,
            (ContractType::NFT, "balanceof-this-pricing") => true,

            _ => false,
        }
    }
}
