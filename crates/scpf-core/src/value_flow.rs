#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueNode {
    MsgValue,
    ExternalDeposit { token: String },
    FlashLoan { token: String, amount: String },
    ContractBalance,
    TokenBalance { token: String },
    Calculation { expression: String },
    EthTransfer { to: String },
    TokenTransfer { token: String, to: String },
}

#[derive(Debug, Clone)]
pub struct ValueEdge {
    pub from: ValueNode,
    pub to: ValueNode,
    pub amount: AmountExpr,
    pub condition: Option<String>,
    pub function: String,
}

#[derive(Debug, Clone)]
pub enum AmountExpr {
    Full,
    Partial {
        numerator: String,
        denominator: String,
    },
    Calculated {
        expression: String,
    },
    UserControlled {
        param: String,
    },
}

#[derive(Debug, Clone)]
pub struct ValueExtractionPath {
    pub entry: ValueNode,
    pub exit: ValueNode,
    pub edges: Vec<ValueEdge>,
    pub extraction_type: ExtractionType,
    pub profit_calculation: String,
    pub exploit_steps: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ExtractionType {
    DirectDrain,
    PriceManipulation,
    FlashLoanArbitrage,
    ReentrancyDrain,
    FeeManipulation,
    RoundingExploit,
}

pub struct ValueFlowAnalyzer {
    edges: Vec<ValueEdge>,
    entry_nodes: Vec<ValueNode>,
    exit_nodes: Vec<ValueNode>,
}

impl ValueFlowAnalyzer {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            entry_nodes: Vec::new(),
            exit_nodes: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, node: ValueNode) {
        self.entry_nodes.push(node);
    }

    pub fn add_exit(&mut self, node: ValueNode) {
        self.exit_nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: ValueEdge) {
        self.edges.push(edge);
    }

    pub fn analyze(&self) -> Vec<ValueExtractionPath> {
        let mut paths = Vec::new();

        for entry in &self.entry_nodes {
            for exit in &self.exit_nodes {
                if let Some(path) = self.find_path(entry, exit) {
                    if let Some(extraction_path) = self.analyze_path(path) {
                        paths.push(extraction_path);
                    }
                }
            }
        }

        paths
    }

    fn find_path(&self, start: &ValueNode, end: &ValueNode) -> Option<Vec<ValueEdge>> {
        let mut path = Vec::new();
        let mut visited = std::collections::HashSet::new();

        if self.dfs(start, end, &mut path, &mut visited) {
            Some(path)
        } else {
            None
        }
    }

    fn dfs(
        &self,
        current: &ValueNode,
        target: &ValueNode,
        path: &mut Vec<ValueEdge>,
        visited: &mut std::collections::HashSet<ValueNode>,
    ) -> bool {
        if current == target {
            return true;
        }

        visited.insert(current.clone());

        for edge in &self.edges {
            if &edge.from == current && !visited.contains(&edge.to) {
                path.push(edge.clone());
                if self.dfs(&edge.to, target, path, visited) {
                    return true;
                }
                path.pop();
            }
        }

        false
    }

    fn analyze_path(&self, edges: Vec<ValueEdge>) -> Option<ValueExtractionPath> {
        if edges.is_empty() {
            return None;
        }

        let entry = edges.first()?.from.clone();
        let exit = edges.last()?.to.clone();

        let extraction_type = self.classify_extraction(&edges);
        let profit = self.calculate_profit_potential(&edges);
        let steps = self.generate_exploit_steps(&edges);

        Some(ValueExtractionPath {
            entry,
            exit,
            edges,
            extraction_type,
            profit_calculation: profit,
            exploit_steps: steps,
        })
    }

    fn classify_extraction(&self, edges: &[ValueEdge]) -> ExtractionType {
        if let Some(first_edge) = edges.first() {
            if matches!(first_edge.from, ValueNode::FlashLoan { .. }) {
                return ExtractionType::FlashLoanArbitrage;
            }
        }

        for edge in edges {
            if let ValueNode::Calculation { expression } = &edge.to {
                if expression.contains("price") || expression.contains("rate") {
                    return ExtractionType::PriceManipulation;
                }
                if expression.contains("/") && expression.contains("*") {
                    return ExtractionType::RoundingExploit;
                }
            }

            if matches!(edge.amount, AmountExpr::UserControlled { .. }) {
                return ExtractionType::DirectDrain;
            }
        }

        ExtractionType::DirectDrain
    }

    fn calculate_profit_potential(&self, edges: &[ValueEdge]) -> String {
        let mut calculation = String::from("Profit = ");

        for (i, edge) in edges.iter().enumerate() {
            match &edge.amount {
                AmountExpr::Full => calculation.push_str("100%"),
                AmountExpr::Partial {
                    numerator,
                    denominator,
                } => {
                    calculation.push_str(&format!("({}/{})", numerator, denominator));
                }
                AmountExpr::Calculated { expression } => {
                    calculation.push_str(&format!("({})", expression));
                }
                AmountExpr::UserControlled { param } => {
                    calculation.push_str(&format!("user_{}", param));
                }
            }

            if i < edges.len() - 1 {
                calculation.push_str(" × ");
            }
        }

        calculation
    }

    fn generate_exploit_steps(&self, edges: &[ValueEdge]) -> Vec<String> {
        let mut steps = Vec::new();

        if let Some(first_edge) = edges.first() {
            match &first_edge.from {
                ValueNode::MsgValue => {
                    steps.push("1. Send ETH to vulnerable function".to_string());
                }
                ValueNode::FlashLoan { token, amount } => {
                    steps.push(format!("1. Take flash loan of {} {}", amount, token));
                }
                ValueNode::ExternalDeposit { token } => {
                    steps.push(format!("1. Deposit {} tokens", token));
                }
                _ => {}
            }
        }

        let mut step_num = 2;
        for edge in edges.iter().skip(1) {
            if let ValueNode::Calculation { expression } = &edge.to {
                steps.push(format!("{}. Trigger calculation: {}", step_num, expression));
                step_num += 1;
            }
        }

        if let Some(last_edge) = edges.last() {
            match &last_edge.to {
                ValueNode::EthTransfer { to } => {
                    steps.push(format!("{}. Receive ETH at {}", step_num, to));
                }
                ValueNode::TokenTransfer { token, to } => {
                    steps.push(format!("{}. Receive {} tokens at {}", step_num, token, to));
                }
                _ => {}
            }
        }

        steps
    }

    pub fn get_high_value_paths(&self) -> Vec<ValueExtractionPath> {
        self.analyze()
            .into_iter()
            .filter(|path| {
                matches!(
                    path.extraction_type,
                    ExtractionType::DirectDrain
                        | ExtractionType::FlashLoanArbitrage
                        | ExtractionType::PriceManipulation
                )
            })
            .collect()
    }

    pub fn export_summary(&self) -> String {
        let paths = self.analyze();
        let mut summary = String::from("# Value Flow Analysis\n\n");

        summary.push_str(&format!("Total paths found: {}\n\n", paths.len()));

        for (i, path) in paths.iter().enumerate() {
            summary.push_str(&format!("## Path {}\n", i + 1));
            summary.push_str(&format!("Type: {:?}\n", path.extraction_type));
            summary.push_str(&format!("Entry: {:?}\n", path.entry));
            summary.push_str(&format!("Exit: {:?}\n", path.exit));
            summary.push_str(&format!("Profit: {}\n\n", path.profit_calculation));

            summary.push_str("### Exploit Steps:\n");
            for step in &path.exploit_steps {
                summary.push_str(&format!("{}\n", step));
            }
            summary.push('\n');
        }

        summary
    }
}

impl Default for ValueFlowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_flow_analyzer() {
        let mut analyzer = ValueFlowAnalyzer::new();

        let entry = ValueNode::MsgValue;
        let exit = ValueNode::EthTransfer {
            to: "attacker".to_string(),
        };

        analyzer.add_entry(entry.clone());
        analyzer.add_exit(exit.clone());

        analyzer.add_edge(ValueEdge {
            from: entry,
            to: exit,
            amount: AmountExpr::Full,
            condition: None,
            function: "withdraw".to_string(),
        });

        let paths = analyzer.analyze();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_extraction_classification() {
        let analyzer = ValueFlowAnalyzer::new();

        let edges = vec![ValueEdge {
            from: ValueNode::FlashLoan {
                token: "USDC".to_string(),
                amount: "1000000".to_string(),
            },
            to: ValueNode::TokenTransfer {
                token: "USDC".to_string(),
                to: "attacker".to_string(),
            },
            amount: AmountExpr::Full,
            condition: None,
            function: "exploit".to_string(),
        }];

        let extraction_type = analyzer.classify_extraction(&edges);
        assert!(matches!(
            extraction_type,
            ExtractionType::FlashLoanArbitrage
        ));
    }
}
