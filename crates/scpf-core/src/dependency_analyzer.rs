use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub contracts: HashMap<String, ContractNode>,
    pub edges: Vec<DependencyEdge>,
    pub external_calls: Vec<ExternalCall>,
    pub inheritance_tree: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ContractNode {
    pub name: String,
    pub file_path: String,
    pub is_interface: bool,
    pub is_library: bool,
    pub functions: Vec<FunctionInfo>,
    pub state_variables: usize,
}

#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    Inherits,
    Calls,
    DelegateCalls,
    Uses,
}

#[derive(Debug, Clone)]
pub struct ExternalCall {
    pub from_contract: String,
    pub from_function: String,
    pub to_contract: String,
    pub call_type: CallType,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub enum CallType {
    Direct,
    LowLevelCall,
    DelegateCall,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub visibility: Visibility,
    pub is_payable: bool,
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    External,
    Internal,
    Private,
}

#[derive(Debug, Clone)]
pub struct AttackSurface {
    pub public_functions: usize,
    pub external_dependencies: usize,
    pub receives_eth: bool,
    pub has_delegatecall: bool,
}

pub struct DependencyAnalyzer {
    contracts: HashMap<String, ContractNode>,
    edges: Vec<DependencyEdge>,
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_contract(&mut self, name: String, node: ContractNode) {
        self.contracts.insert(name, node);
    }

    pub fn add_edge(&mut self, from: String, to: String, edge_type: EdgeType) {
        self.edges.push(DependencyEdge {
            from,
            to,
            edge_type,
        });
    }

    pub fn build_graph(&self) -> DependencyGraph {
        let inheritance_tree = self.build_inheritance_tree();
        let external_calls = self.extract_external_calls();

        DependencyGraph {
            contracts: self.contracts.clone(),
            edges: self.edges.clone(),
            external_calls,
            inheritance_tree,
        }
    }

    fn build_inheritance_tree(&self) -> HashMap<String, Vec<String>> {
        let mut tree = HashMap::new();

        for edge in &self.edges {
            if matches!(edge.edge_type, EdgeType::Inherits) {
                tree.entry(edge.from.clone())
                    .or_insert_with(Vec::new)
                    .push(edge.to.clone());
            }
        }

        tree
    }

    fn extract_external_calls(&self) -> Vec<ExternalCall> {
        let mut calls = Vec::new();

        for edge in &self.edges {
            if matches!(edge.edge_type, EdgeType::Calls | EdgeType::DelegateCalls) {
                calls.push(ExternalCall {
                    from_contract: edge.from.clone(),
                    from_function: String::new(),
                    to_contract: edge.to.clone(),
                    call_type: match edge.edge_type {
                        EdgeType::DelegateCalls => CallType::DelegateCall,
                        _ => CallType::Direct,
                    },
                    line_number: 0,
                });
            }
        }

        calls
    }

    pub fn get_attack_surface(&self, contract: &str) -> AttackSurface {
        let node = self.contracts.get(contract);

        let public_functions = node
            .map(|n| {
                n.functions
                    .iter()
                    .filter(|f| matches!(f.visibility, Visibility::Public | Visibility::External))
                    .count()
            })
            .unwrap_or(0);

        let external_dependencies = self
            .edges
            .iter()
            .filter(|e| e.from == contract && matches!(e.edge_type, EdgeType::Calls))
            .count();

        let receives_eth = node
            .map(|n| n.functions.iter().any(|f| f.is_payable))
            .unwrap_or(false);

        let has_delegatecall = self
            .edges
            .iter()
            .any(|e| e.from == contract && matches!(e.edge_type, EdgeType::DelegateCalls));

        AttackSurface {
            public_functions,
            external_dependencies,
            receives_eth,
            has_delegatecall,
        }
    }

    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for contract in self.contracts.keys() {
            if !visited.contains(contract) {
                self.dfs_cycle(
                    contract,
                    &mut visited,
                    &mut rec_stack,
                    &mut Vec::new(),
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        for edge in &self.edges {
            if edge.from == node {
                if !visited.contains(&edge.to) {
                    self.dfs_cycle(&edge.to, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(&edge.to) {
                    cycles.push(path.clone());
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    pub fn export_graphviz(&self) -> String {
        let mut dot = String::from("digraph Dependencies {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box];\n\n");

        for edge in &self.edges {
            let style = match edge.edge_type {
                EdgeType::Inherits => "solid",
                EdgeType::Calls => "dashed",
                EdgeType::DelegateCalls => "bold,color=red",
                EdgeType::Uses => "dotted",
            };
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [style={}];\n",
                edge.from, edge.to, style
            ));
        }

        dot.push_str("}\n");
        dot
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_analyzer() {
        let mut analyzer = DependencyAnalyzer::new();

        analyzer.add_contract(
            "ContractA".to_string(),
            ContractNode {
                name: "ContractA".to_string(),
                file_path: "a.sol".to_string(),
                is_interface: false,
                is_library: false,
                functions: vec![],
                state_variables: 0,
            },
        );

        analyzer.add_edge(
            "ContractA".to_string(),
            "ContractB".to_string(),
            EdgeType::Inherits,
        );

        let graph = analyzer.build_graph();
        assert_eq!(graph.contracts.len(), 1);
        assert_eq!(graph.edges.len(), 1);
    }
}
