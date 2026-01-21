use scpf_types::*;
use tree_sitter::{Node, Tree};

pub struct SymbolCollector<'a> {
    source: &'a str,
    tree: &'a Tree,
}

impl<'a> SymbolCollector<'a> {
    pub fn new(source: &'a str, tree: &'a Tree) -> Self {
        Self { source, tree }
    }

    pub fn collect(&self) -> ContractContext {
        let mut ctx = ContractContext::default();

        self.collect_functions(&mut ctx);
        self.collect_modifiers(&mut ctx);
        self.collect_state_variables(&mut ctx);

        ctx
    }

    fn collect_functions(&self, ctx: &mut ContractContext) {
        let root = self.tree.root_node();
        self.collect_functions_recursive(root, ctx);
    }

    fn collect_functions_recursive(&self, node: Node, ctx: &mut ContractContext) {
        if node.kind() == "function_definition" {
            if let Some(func_ctx) = self.analyze_function(node) {
                ctx.functions.insert(func_ctx.name.clone(), func_ctx);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_functions_recursive(child, ctx);
        }
    }

    fn analyze_function(&self, node: Node) -> Option<FunctionContext> {
        let name = self.get_function_name(node)?;
        let visibility = self.get_visibility(node);
        let mutability = self.get_mutability(node);
        let modifiers = self.get_function_modifiers(node);
        let start_line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;

        Some(FunctionContext {
            name,
            visibility,
            mutability,
            modifiers,
            external_calls: Vec::new(),
            state_changes: Vec::new(),
            protections: ProtectionSet::default(),
            start_line,
            end_line,
        })
    }

    fn get_function_name(&self, node: Node) -> Option<String> {
        node.child_by_field_name("name")
            .map(|n| self.node_text(n).to_string())
    }

    fn get_visibility(&self, node: Node) -> Visibility {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility" {
                return match self.node_text(child) {
                    "public" => Visibility::Public,
                    "external" => Visibility::External,
                    "internal" => Visibility::Internal,
                    "private" => Visibility::Private,
                    _ => Visibility::Public,
                };
            }
        }
        Visibility::Public
    }

    fn get_mutability(&self, node: Node) -> Mutability {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "state_mutability" {
                return match self.node_text(child) {
                    "pure" => Mutability::Pure,
                    "view" => Mutability::View,
                    "payable" => Mutability::Payable,
                    _ => Mutability::NonPayable,
                };
            }
        }
        Mutability::NonPayable
    }

    fn get_function_modifiers(&self, node: Node) -> Vec<String> {
        let mut modifiers = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "modifier_invocation" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    modifiers.push(self.node_text(name_node).to_string());
                } else {
                    // Try getting text directly
                    let text = self.node_text(child);
                    // Extract identifier from text like "nonReentrant()" -> "nonReentrant"
                    if let Some(name) = text.split('(').next() {
                        modifiers.push(name.trim().to_string());
                    }
                }
            }
        }

        modifiers
    }

    fn collect_modifiers(&self, ctx: &mut ContractContext) {
        let root = self.tree.root_node();
        self.collect_modifiers_recursive(root, ctx);
    }

    fn collect_modifiers_recursive(&self, node: Node, ctx: &mut ContractContext) {
        if node.kind() == "modifier_definition" {
            if let Some(mod_ctx) = self.analyze_modifier(node) {
                ctx.modifiers.insert(mod_ctx.name.clone(), mod_ctx);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_modifiers_recursive(child, ctx);
        }
    }

    fn analyze_modifier(&self, node: Node) -> Option<ModifierContext> {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(n).to_string())?;

        Some(ModifierContext {
            name,
            modifier_type: ModifierType::Unknown,
            confidence: 0.0,
            has_state_check: self.has_require_or_revert(node),
            can_revert: self.has_require_or_revert(node),
        })
    }

    fn has_require_or_revert(&self, node: Node) -> bool {
        let text = self.node_text(node);
        text.contains("require(") || text.contains("revert(") || text.contains("assert(")
    }

    fn collect_state_variables(&self, ctx: &mut ContractContext) {
        let root = self.tree.root_node();
        self.collect_state_variables_recursive(root, ctx);
    }

    fn collect_state_variables_recursive(&self, node: Node, ctx: &mut ContractContext) {
        if node.kind() == "state_variable_declaration" {
            if let Some(var) = self.analyze_state_variable(node) {
                ctx.state_variables.insert(var.name.clone(), var);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_state_variables_recursive(child, ctx);
        }
    }

    fn analyze_state_variable(&self, node: Node) -> Option<StateVariable> {
        let type_node = node.child_by_field_name("type")?;
        let name_node = node.child_by_field_name("name")?;

        Some(StateVariable {
            name: self.node_text(name_node).to_string(),
            var_type: self.node_text(type_node).to_string(),
        })
    }

    fn node_text(&self, node: Node) -> &str {
        &self.source[node.byte_range()]
    }
}
