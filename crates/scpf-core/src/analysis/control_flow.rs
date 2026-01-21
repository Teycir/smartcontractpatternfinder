use tree_sitter::{Node, Tree};

/// Analyze if reentrancy vulnerability exists based on control flow
/// Returns true if VULNERABLE (state change AFTER external call)
pub fn is_vulnerable_reentrancy(tree: &Tree, source: &str, line: usize) -> bool {
    let root = tree.root_node();

    // Find function containing this line
    let func_node = find_function_at_line(root, source, line);
    if func_node.is_none() {
        return true; // Conservative: report if can't analyze
    }

    let func = func_node.unwrap();

    // Find all external calls and state changes
    let external_calls = find_external_calls(func, source);
    let state_changes = find_state_changes(func, source);

    // Check if ANY state change happens AFTER external call
    for call_line in &external_calls {
        for change_line in &state_changes {
            if change_line > call_line {
                return true; // VULNERABLE: state change after call
            }
        }
    }

    false // SAFE: no state changes after calls
}

fn find_function_at_line<'a>(node: Node<'a>, source: &str, target_line: usize) -> Option<Node<'a>> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "function_definition" {
            let start_line = child.start_position().row + 1;
            let end_line = child.end_position().row + 1;

            if start_line <= target_line && target_line <= end_line {
                return Some(child);
            }
        }

        if let Some(found) = find_function_at_line(child, source, target_line) {
            return Some(found);
        }
    }

    None
}

fn find_external_calls(func: Node, source: &str) -> Vec<usize> {
    let mut calls = Vec::new();
    collect_external_calls(func, source, &mut calls);
    calls
}

fn collect_external_calls(node: Node, source: &str, calls: &mut Vec<usize>) {
    let text = node.utf8_text(source.as_bytes()).unwrap_or("");

    // Detect external calls: .call, .delegatecall, .transfer, .send
    if node.kind() == "call_expression" {
        if text.contains(".call")
            || text.contains(".delegatecall")
            || text.contains(".transfer")
            || text.contains(".send")
        {
            calls.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_external_calls(child, source, calls);
    }
}

fn find_state_changes(func: Node, source: &str) -> Vec<usize> {
    let mut changes = Vec::new();
    collect_state_changes(func, source, &mut changes);
    changes
}

fn collect_state_changes(node: Node, source: &str, changes: &mut Vec<usize>) {
    // Detect state changes: assignments, +=, -=, delete
    if node.kind() == "assignment_expression"
        || node.kind() == "augmented_assignment_expression"
        || node.kind() == "delete_statement"
    {
        changes.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_state_changes(child, source, changes);
    }
}
