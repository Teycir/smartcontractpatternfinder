use std::collections::HashMap;
use tree_sitter::{Node, Tree};

/// Trait for dataflow analyzers
pub trait DataFlowAnalyzer {
    fn analyze(&self, tree: &Tree, source: &str) -> Vec<AnalyzerFinding>;
    fn analyzer_id(&self) -> &str;
}

/// Generic dataflow finding from analyzers
#[derive(Debug, Clone)]
pub struct AnalyzerFinding {
    pub analyzer_id: String,
    pub pattern_id: String,
    pub line: usize,
    pub message: String,
    pub severity: DataFlowSeverity,
    pub context: String,
}

#[derive(Debug, Clone, Copy)]
pub enum DataFlowSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Registry for dataflow analyzers
pub struct DataFlowRegistry {
    analyzers: Vec<Box<dyn DataFlowAnalyzer>>,
}

impl DataFlowRegistry {
    pub fn new() -> Self {
        Self {
            analyzers: Vec::new(),
        }
    }

    pub fn register(&mut self, analyzer: Box<dyn DataFlowAnalyzer>) {
        self.analyzers.push(analyzer);
    }

    pub fn analyze_all(&self, tree: &Tree, source: &str) -> Vec<AnalyzerFinding> {
        self.analyzers
            .iter()
            .flat_map(|analyzer| analyzer.analyze(tree, source))
            .collect()
    }

    pub fn with_default_analyzers() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(ReentrancyAnalyzer));
        registry
    }
}

/// Reentrancy analyzer implementation
pub struct ReentrancyAnalyzer;

impl DataFlowAnalyzer for ReentrancyAnalyzer {
    fn analyze(&self, tree: &Tree, source: &str) -> Vec<AnalyzerFinding> {
        let analysis = DataFlowAnalysis::analyze(tree, source);
        analysis
            .reentrancy_risks
            .into_iter()
            .map(|risk| AnalyzerFinding {
                analyzer_id: self.analyzer_id().to_string(),
                pattern_id: "state-mutation-after-call".to_string(),
                line: risk.call_line,
                message: format!(
                    "Data flow analysis: {} call on line {} followed by state mutation of '{}' on line {}. Potential reentrancy vulnerability.",
                    risk.call_method, risk.call_line, risk.state_var, risk.state_change_line
                ),
                severity: match risk.severity {
                    RiskSeverity::Critical => DataFlowSeverity::Critical,
                    RiskSeverity::High => DataFlowSeverity::High,
                    RiskSeverity::Medium => DataFlowSeverity::Medium,
                },
                context: risk.call_method.clone(),
            })
            .collect()
    }

    fn analyzer_id(&self) -> &str {
        "dataflow-reentrancy"
    }
}

/// Type of state mutation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MutationType {
    DirectAssignment,
    MappingWrite,
    ArrayWrite,
    ArrayPush,
    ArrayPop,
    StructFieldWrite,
    Increment,
    Decrement,
    CompoundAssignment,
    Delete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateChange {
    Assignment { var: String, line: usize },
    MapWrite { map: String, line: usize },
    ArrayWrite { array: String, line: usize },
    Increment { var: String, line: usize },
    Decrement { var: String, line: usize },
}

#[derive(Debug, Clone)]
pub struct ExternalCall {
    pub method: String,
    pub line: usize,
    pub has_value: bool,
}

#[derive(Debug)]
pub struct DataFlowAnalysis {
    pub external_calls: Vec<ExternalCall>,
    pub state_changes: Vec<StateChange>,
    pub reentrancy_risks: Vec<ReentrancyRisk>,
}

#[derive(Debug, Clone)]
pub struct ReentrancyRisk {
    pub call_line: usize,
    pub call_method: String,
    pub state_change_line: usize,
    pub state_var: String,
    pub severity: RiskSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskSeverity {
    Critical, // Value transfer + balance update after
    High,     // External call + state change after
    Medium,   // Potential issue
}

/// State mutation tracker
pub struct StateMutationTracker {
    pub state_variables: HashMap<String, StateVariable>,
    pub mutations: Vec<MutationEvent>,
    pub external_calls: HashMap<String, Vec<usize>>,
    current_function: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StateVariable {
    pub name: String,
    pub var_type: String,
    pub declaration_line: usize,
}

#[derive(Debug, Clone)]
pub struct MutationEvent {
    pub variable: String,
    pub mutation_type: MutationType,
    pub function_name: String,
    pub line: usize,
    pub after_external_call: bool,
}

impl StateMutationTracker {
    pub fn new() -> Self {
        Self {
            state_variables: HashMap::new(),
            mutations: Vec::new(),
            external_calls: HashMap::new(),
            current_function: None,
        }
    }

    pub fn analyze_contract(&mut self, root: Node, source: &str) {
        self.collect_state_variables(root, source);
        self.analyze_functions(root, source);
        self.correlate_calls_and_mutations();
    }

    fn collect_state_variables(&mut self, node: Node, source: &str) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "state_variable_declaration" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    let var_type = child
                        .child_by_field_name("type")
                        .and_then(|t| t.utf8_text(source.as_bytes()).ok())
                        .unwrap_or("unknown")
                        .to_string();
                    self.state_variables.insert(
                        name.clone(),
                        StateVariable {
                            name,
                            var_type,
                            declaration_line: child.start_position().row + 1,
                        },
                    );
                }
            }
            self.collect_state_variables(child, source);
        }
    }

    fn analyze_functions(&mut self, node: Node, source: &str) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_definition" {
                let func_name = child
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("anonymous")
                    .to_string();
                self.current_function = Some(func_name.clone());
                self.find_external_calls(child, source);
                self.find_mutations(child, source);
                self.current_function = None;
            }
            self.analyze_functions(child, source);
        }
    }

    fn find_external_calls(&mut self, node: Node, source: &str) {
        let func_name = self.current_function.clone().unwrap_or_default();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_call" {
                let text = child.utf8_text(source.as_bytes()).unwrap_or("");
                if text.contains(".call")
                    || text.contains(".delegatecall")
                    || text.contains(".send")
                {
                    let line = child.start_position().row + 1;
                    self.external_calls
                        .entry(func_name.clone())
                        .or_insert_with(Vec::new)
                        .push(line);
                }
            }
            self.find_external_calls(child, source);
        }
    }

    fn find_mutations(&mut self, node: Node, source: &str) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "assignment" => {
                    if let Some(event) = self.parse_assignment(child, source) {
                        self.mutations.push(event);
                    }
                }
                "augmented_assignment" => {
                    if let Some(event) = self.parse_augmented(child, source) {
                        self.mutations.push(event);
                    }
                }
                _ => {}
            }
            self.find_mutations(child, source);
        }
    }

    fn parse_assignment(&self, node: Node, source: &str) -> Option<MutationEvent> {
        let lhs = node.child(0)?;
        let lhs_text = lhs.utf8_text(source.as_bytes()).ok()?.to_string();
        let (var_name, mutation_type) = self.classify_lhs(&lhs_text);
        if !self.state_variables.contains_key(&var_name) {
            return None;
        }
        Some(MutationEvent {
            variable: var_name,
            mutation_type,
            function_name: self.current_function.clone().unwrap_or_default(),
            line: node.start_position().row + 1,
            after_external_call: false,
        })
    }

    fn parse_augmented(&self, node: Node, source: &str) -> Option<MutationEvent> {
        let lhs = node.child(0)?;
        let var = lhs.utf8_text(source.as_bytes()).ok()?.to_string();
        if !self.state_variables.contains_key(&var) {
            return None;
        }
        Some(MutationEvent {
            variable: var,
            mutation_type: MutationType::CompoundAssignment,
            function_name: self.current_function.clone().unwrap_or_default(),
            line: node.start_position().row + 1,
            after_external_call: false,
        })
    }

    fn classify_lhs(&self, lhs: &str) -> (String, MutationType) {
        if lhs.contains('[') {
            let base = lhs.split('[').next().unwrap_or(lhs).trim().to_string();
            return (base, MutationType::MappingWrite);
        }
        (lhs.trim().to_string(), MutationType::DirectAssignment)
    }

    fn correlate_calls_and_mutations(&mut self) {
        for mutation in &mut self.mutations {
            if let Some(call_lines) = self.external_calls.get(&mutation.function_name) {
                mutation.after_external_call = call_lines
                    .iter()
                    .any(|&call_line| mutation.line > call_line);
            }
        }
    }

    pub fn get_cei_violations(&self) -> Vec<&MutationEvent> {
        self.mutations
            .iter()
            .filter(|m| m.after_external_call)
            .collect()
    }
}

impl DataFlowAnalysis {
    pub fn analyze(tree: &Tree, source: &str) -> Self {
        let root = tree.root_node();
        let mut external_calls = Vec::new();
        let mut state_changes = Vec::new();

        Self::traverse_node(root, source, &mut external_calls, &mut state_changes);

        let reentrancy_risks = Self::detect_reentrancy(&external_calls, &state_changes);

        Self {
            external_calls,
            state_changes,
            reentrancy_risks,
        }
    }

    fn traverse_node(
        node: Node,
        source: &str,
        calls: &mut Vec<ExternalCall>,
        changes: &mut Vec<StateChange>,
    ) {
        match node.kind() {
            "function_call" => {
                if let Some(call) = Self::extract_external_call(node, source) {
                    calls.push(call);
                }
            }
            "assignment" => {
                if let Some(change) = Self::extract_state_change(node, source) {
                    changes.push(change);
                }
            }
            "augmented_assignment" => {
                if let Some(change) = Self::extract_augmented_assignment(node, source) {
                    changes.push(change);
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::traverse_node(child, source, calls, changes);
        }
    }

    fn extract_external_call(node: Node, source: &str) -> Option<ExternalCall> {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        if children.is_empty() {
            return None;
        }

        let first_child = children[0];
        if first_child.kind() == "member_access" {
            let method = Self::get_method_name(first_child, source)?;
            if matches!(
                method.as_str(),
                "call" | "delegatecall" | "staticcall" | "send" | "transfer"
            ) {
                let has_value = Self::has_value_option(node, source);
                return Some(ExternalCall {
                    method,
                    line: node.start_position().row + 1,
                    has_value,
                });
            }
        }

        None
    }

    fn get_method_name(node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(child.utf8_text(source.as_bytes()).ok()?.to_string());
            }
        }
        None
    }

    fn has_value_option(node: Node, source: &str) -> bool {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        text.contains("value:")
    }

    fn extract_state_change(node: Node, source: &str) -> Option<StateChange> {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        if children.is_empty() {
            return None;
        }

        let left = children[0];
        let line = node.start_position().row + 1;

        match left.kind() {
            "identifier" => {
                let var = left.utf8_text(source.as_bytes()).ok()?.to_string();
                Some(StateChange::Assignment { var, line })
            }
            "member_access" => {
                let var = left.utf8_text(source.as_bytes()).ok()?.to_string();
                if var.contains('[') {
                    Some(StateChange::MapWrite {
                        map: var.split('[').next()?.to_string(),
                        line,
                    })
                } else {
                    Some(StateChange::Assignment { var, line })
                }
            }
            _ => None,
        }
    }

    fn extract_augmented_assignment(node: Node, source: &str) -> Option<StateChange> {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        if children.is_empty() {
            return None;
        }

        let left = children[0];
        let line = node.start_position().row + 1;
        let var = left.utf8_text(source.as_bytes()).ok()?.to_string();

        let text = node.utf8_text(source.as_bytes()).ok()?;
        if text.contains("+=") {
            Some(StateChange::Increment { var, line })
        } else if text.contains("-=") {
            Some(StateChange::Decrement { var, line })
        } else {
            Some(StateChange::Assignment { var, line })
        }
    }

    fn detect_reentrancy(calls: &[ExternalCall], changes: &[StateChange]) -> Vec<ReentrancyRisk> {
        let mut risks = Vec::new();

        for call in calls {
            for change in changes {
                let change_line = match change {
                    StateChange::Assignment { line, .. }
                    | StateChange::MapWrite { line, .. }
                    | StateChange::ArrayWrite { line, .. }
                    | StateChange::Increment { line, .. }
                    | StateChange::Decrement { line, .. } => *line,
                };

                if change_line > call.line && (change_line - call.line) < 50 {
                    let state_var = match change {
                        StateChange::Assignment { var, .. }
                        | StateChange::MapWrite { map: var, .. }
                        | StateChange::ArrayWrite { array: var, .. }
                        | StateChange::Increment { var, .. }
                        | StateChange::Decrement { var, .. } => var.clone(),
                    };

                    let severity = if call.has_value
                        && matches!(
                            change,
                            StateChange::MapWrite { .. } | StateChange::Decrement { .. }
                        ) {
                        RiskSeverity::Critical
                    } else if call.method == "call" || call.method == "delegatecall" {
                        RiskSeverity::High
                    } else {
                        RiskSeverity::Medium
                    };

                    risks.push(ReentrancyRisk {
                        call_line: call.line,
                        call_method: call.method.clone(),
                        state_change_line: change_line,
                        state_var,
                        severity,
                    });
                }
            }
        }

        risks
    }
}
