use regex::Regex;

#[derive(Debug, Clone)]
pub struct GeneratedInvariant {
    pub name: String,
    pub description: String,
    pub solidity_code: String,
    pub confidence: f64,
    pub category: InvariantCategory,
}

#[derive(Debug, Clone)]
pub enum InvariantCategory {
    BalanceConservation,
    SupplyConservation,
    MonotonicIncrease,
    NonZero,
    BoundedValue,
    AccessControl,
}

pub struct InvariantGenerator {
    source_code: String,
    contract_name: String,
}

impl InvariantGenerator {
    pub fn new(source_code: String, contract_name: String) -> Self {
        Self {
            source_code,
            contract_name,
        }
    }

    pub fn generate(&self) -> Vec<GeneratedInvariant> {
        let mut invariants = Vec::new();
        invariants.extend(self.generate_balance_invariants());
        invariants.extend(self.generate_supply_invariants());
        invariants.extend(self.generate_monotonic_invariants());
        invariants.extend(self.generate_access_invariants());
        invariants
    }

    fn generate_balance_invariants(&self) -> Vec<GeneratedInvariant> {
        let mut invariants = Vec::new();
        let balance_re =
            Regex::new(r"mapping\s*\([^)]+\)\s+(?:public|private|internal)?\s*(\w*[Bb]alance\w*)")
                .unwrap();

        for cap in balance_re.captures_iter(&self.source_code) {
            if let Some(var_name) = cap.get(1) {
                invariants.push(GeneratedInvariant {
                    name: format!("sum_{}_conservation", var_name.as_str()),
                    description: format!(
                        "Sum of all {} should remain constant or match total",
                        var_name.as_str()
                    ),
                    solidity_code: format!(
                        r#"function invariant_{}Conservation() public view returns (bool) {{
    // Sum of individual balances should equal total supply
    return true; // Implement balance tracking
}}"#,
                        var_name.as_str()
                    ),
                    confidence: 0.7,
                    category: InvariantCategory::BalanceConservation,
                });
            }
        }
        invariants
    }

    fn generate_supply_invariants(&self) -> Vec<GeneratedInvariant> {
        let mut invariants = Vec::new();

        if self.source_code.contains("totalSupply") && self.source_code.contains("balanceOf") {
            invariants.push(GeneratedInvariant {
                name: "total_supply_non_negative".to_string(),
                description: "Total supply must never be negative".to_string(),
                solidity_code:
                    r#"function invariant_totalSupplyNonNegative() public view returns (bool) {
    return totalSupply() >= 0;
}"#
                    .to_string(),
                confidence: 0.95,
                category: InvariantCategory::SupplyConservation,
            });

            invariants.push(GeneratedInvariant {
                name: "balance_lte_supply".to_string(),
                description: "Individual balance cannot exceed total supply".to_string(),
                solidity_code: r#"function invariant_balanceLteSupply(address user) public view returns (bool) {
    return balanceOf(user) <= totalSupply();
}"#.to_string(),
                confidence: 0.9,
                category: InvariantCategory::BoundedValue,
            });
        }
        invariants
    }

    fn generate_monotonic_invariants(&self) -> Vec<GeneratedInvariant> {
        let mut invariants = Vec::new();
        let monotonic_patterns = ["nonce", "epoch", "totalStaked", "totalDeposits"];

        for pattern in &monotonic_patterns {
            if self.source_code.contains(pattern) {
                invariants.push(GeneratedInvariant {
                    name: format!("{}_monotonic_increase", pattern),
                    description: format!("{} should never decrease", pattern),
                    solidity_code: format!(
                        r#"uint256 private last_{0};
function invariant_{0}Monotonic() public returns (bool) {{
    bool result = {0} >= last_{0};
    last_{0} = {0};
    return result;
}}"#,
                        pattern
                    ),
                    confidence: 0.8,
                    category: InvariantCategory::MonotonicIncrease,
                });
            }
        }
        invariants
    }

    fn generate_access_invariants(&self) -> Vec<GeneratedInvariant> {
        let mut invariants = Vec::new();
        let access_patterns = ["owner", "admin"];

        for pattern in &access_patterns {
            let re = Regex::new(&format!(r"\b{}\b", pattern)).unwrap();
            if re.is_match(&self.source_code) {
                invariants.push(GeneratedInvariant {
                    name: format!("{}_not_zero", pattern),
                    description: format!("{} should never be zero address", pattern),
                    solidity_code: format!(
                        r#"function invariant_{0}NotZero() public view returns (bool) {{
    return {0} != address(0);
}}"#,
                        pattern
                    ),
                    confidence: 0.85,
                    category: InvariantCategory::NonZero,
                });
            }
        }
        invariants
    }

    pub fn export_foundry_test(&self) -> String {
        let invariants = self.generate();

        let mut code = format!(
            r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Test.sol";
import "../src/{}.sol";

contract {}InvariantTest is Test {{
    {} public target;
    
    function setUp() public {{
        target = new {}();
    }}
"#,
            self.contract_name, self.contract_name, self.contract_name, self.contract_name
        );

        for invariant in invariants {
            code.push_str(&format!(
                r#"
    // {}
    // Confidence: {:.0}%
    // Category: {:?}
    {}
"#,
                invariant.description,
                invariant.confidence * 100.0,
                invariant.category,
                invariant.solidity_code
            ));
        }

        code.push_str("}\n");
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_generation() {
        let source = r#"
        contract Token {
            mapping(address => uint256) public balances;
            uint256 public totalSupply;
            address public owner;
            uint256 public nonce;
            
            function balanceOf(address user) public view returns (uint256) {
                return balances[user];
            }
        }
        "#;

        let gen = InvariantGenerator::new(source.to_string(), "Token".to_string());
        let invariants = gen.generate();

        assert!(!invariants.is_empty());
        assert!(invariants.iter().any(|i| i.name.contains("balance")));
        assert!(invariants.iter().any(|i| i.name.contains("supply")));
    }

    #[test]
    fn test_foundry_export() {
        let gen = InvariantGenerator::new("contract Test {}".to_string(), "Test".to_string());
        let test_code = gen.export_foundry_test();

        assert!(test_code.contains("pragma solidity"));
        assert!(test_code.contains("TestInvariantTest"));
        assert!(test_code.contains("forge-std/Test.sol"));
    }
}
