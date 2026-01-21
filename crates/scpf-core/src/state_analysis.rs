use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ContractState {
    pub storage: HashMap<String, StorageSlot>,
    pub balances: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct StorageSlot {
    pub name: String,
    pub slot_type: SlotType,
    pub read_by: Vec<String>,
    pub written_by: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SlotType {
    Scalar {
        type_name: String,
    },
    Mapping {
        key_type: String,
        value_type: String,
    },
    Array {
        element_type: String,
    },
}

#[derive(Debug, Clone)]
pub struct Invariant {
    pub description: String,
    pub expression: String,
    pub severity: InvariantSeverity,
}

#[derive(Debug, Clone)]
pub enum InvariantSeverity {
    Critical,
    High,
    Medium,
}

#[derive(Debug, Clone)]
pub struct StateViolation {
    pub invariant: Invariant,
    pub attack_sequence: Vec<FunctionCall>,
    pub exploit_description: String,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub function: String,
    pub caller: String,
    pub arguments: Vec<String>,
}

pub struct StateAnalyzer {
    invariants: Vec<Invariant>,
    violations: Vec<StateViolation>,
}

impl StateAnalyzer {
    pub fn new() -> Self {
        Self {
            invariants: Vec::new(),
            violations: Vec::new(),
        }
    }

    pub fn add_invariant(&mut self, invariant: Invariant) {
        self.invariants.push(invariant);
    }

    pub fn analyze(&mut self, state: &ContractState) -> Vec<StateViolation> {
        self.violations.clear();

        let common_invariants = self.infer_common_invariants(state);
        self.invariants.extend(common_invariants);

        for invariant in &self.invariants.clone() {
            if let Some(violation) = self.check_invariant(state, invariant) {
                self.violations.push(violation);
            }
        }

        self.violations.clone()
    }

    fn infer_common_invariants(&self, state: &ContractState) -> Vec<Invariant> {
        let mut invariants = Vec::new();

        for (name, slot) in &state.storage {
            if name.contains("owner") || name.contains("admin") {
                invariants.push(Invariant {
                    description: format!("{} should never be zero address", name),
                    expression: format!("{} != address(0)", name),
                    severity: InvariantSeverity::Critical,
                });
            }

            if name.contains("balance") {
                invariants.push(Invariant {
                    description: format!("{} should never exceed total supply", name),
                    expression: format!("{}[user] <= totalSupply", name),
                    severity: InvariantSeverity::High,
                });
            }

            if name.contains("nonce") || name.contains("counter") {
                invariants.push(Invariant {
                    description: format!("{} should only increase", name),
                    expression: format!("{}_new >= {}_old", name, name),
                    severity: InvariantSeverity::Medium,
                });
            }

            if matches!(slot.slot_type, SlotType::Mapping { .. })
                && !slot.written_by.is_empty()
                && slot.written_by.len() > 1
            {
                invariants.push(Invariant {
                    description: format!(
                        "{} has multiple writers - check for race conditions",
                        name
                    ),
                    expression: format!("Concurrent writes to {}", name),
                    severity: InvariantSeverity::High,
                });
            }
        }

        invariants
    }

    fn check_invariant(
        &self,
        state: &ContractState,
        invariant: &Invariant,
    ) -> Option<StateViolation> {
        let attack_sequences = self.generate_attack_sequences(state, invariant);

        if !attack_sequences.is_empty() {
            Some(StateViolation {
                invariant: invariant.clone(),
                attack_sequence: attack_sequences[0].clone(),
                exploit_description: self.describe_exploit(&attack_sequences[0], invariant),
            })
        } else {
            None
        }
    }

    fn generate_attack_sequences(
        &self,
        _state: &ContractState,
        invariant: &Invariant,
    ) -> Vec<Vec<FunctionCall>> {
        let mut sequences = Vec::new();

        if invariant.description.contains("reentrancy") {
            sequences.push(vec![
                FunctionCall {
                    function: "withdraw".to_string(),
                    caller: "attacker".to_string(),
                    arguments: vec!["amount".to_string()],
                },
                FunctionCall {
                    function: "withdraw".to_string(),
                    caller: "attacker".to_string(),
                    arguments: vec!["amount".to_string()],
                },
            ]);
        }

        if invariant.description.contains("owner") || invariant.description.contains("admin") {
            sequences.push(vec![FunctionCall {
                function: "transferOwnership".to_string(),
                caller: "attacker".to_string(),
                arguments: vec!["address(0)".to_string()],
            }]);
        }

        if invariant.description.contains("balance") {
            sequences.push(vec![FunctionCall {
                function: "mint".to_string(),
                caller: "attacker".to_string(),
                arguments: vec!["MAX_UINT256".to_string()],
            }]);
        }

        sequences
    }

    fn describe_exploit(&self, sequence: &[FunctionCall], invariant: &Invariant) -> String {
        let mut description = String::new();

        description.push_str(&format!(
            "## Invariant Violation: {}\n\n",
            invariant.description
        ));
        description.push_str(&format!("**Severity**: {:?}\n\n", invariant.severity));
        description.push_str("### Attack Sequence:\n\n");

        for (i, call) in sequence.iter().enumerate() {
            description.push_str(&format!(
                "{}. `{}({})` called by {}\n",
                i + 1,
                call.function,
                call.arguments.join(", "),
                call.caller
            ));
        }

        description.push_str(&format!(
            "\n### Impact:\nViolates: `{}`\n",
            invariant.expression
        ));

        description
    }

    pub fn get_violations(&self) -> &[StateViolation] {
        &self.violations
    }

    pub fn get_critical_violations(&self) -> Vec<&StateViolation> {
        self.violations
            .iter()
            .filter(|v| matches!(v.invariant.severity, InvariantSeverity::Critical))
            .collect()
    }

    pub fn export_report(&self) -> String {
        let mut report = String::from("# State Analysis Report\n\n");

        report.push_str(&format!(
            "Total invariants checked: {}\n",
            self.invariants.len()
        ));
        report.push_str(&format!("Violations found: {}\n\n", self.violations.len()));

        let critical = self.get_critical_violations();
        if !critical.is_empty() {
            report.push_str(&format!("## Critical Violations ({})\n\n", critical.len()));
            for violation in critical {
                report.push_str(&violation.exploit_description);
                report.push_str("\n---\n\n");
            }
        }

        let high: Vec<_> = self
            .violations
            .iter()
            .filter(|v| matches!(v.invariant.severity, InvariantSeverity::High))
            .collect();

        if !high.is_empty() {
            report.push_str(&format!("## High Severity Violations ({})\n\n", high.len()));
            for violation in high {
                report.push_str(&violation.exploit_description);
                report.push_str("\n---\n\n");
            }
        }

        report
    }
}

impl Default for StateAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_analyzer() {
        let mut analyzer = StateAnalyzer::new();

        let mut state = ContractState {
            storage: HashMap::new(),
            balances: HashMap::new(),
        };

        state.storage.insert(
            "owner".to_string(),
            StorageSlot {
                name: "owner".to_string(),
                slot_type: SlotType::Scalar {
                    type_name: "address".to_string(),
                },
                read_by: vec!["onlyOwner".to_string()],
                written_by: vec!["transferOwnership".to_string()],
            },
        );

        let violations = analyzer.analyze(&state);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_invariant_inference() {
        let analyzer = StateAnalyzer::new();

        let mut state = ContractState {
            storage: HashMap::new(),
            balances: HashMap::new(),
        };

        state.storage.insert(
            "balances".to_string(),
            StorageSlot {
                name: "balances".to_string(),
                slot_type: SlotType::Mapping {
                    key_type: "address".to_string(),
                    value_type: "uint256".to_string(),
                },
                read_by: vec!["balanceOf".to_string()],
                written_by: vec!["transfer".to_string(), "mint".to_string()],
            },
        );

        let invariants = analyzer.infer_common_invariants(&state);
        assert!(!invariants.is_empty());
    }
}
