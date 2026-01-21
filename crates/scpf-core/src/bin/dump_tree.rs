use std::env;
use std::fs;
use tree_sitter::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: dump_tree <solidity_file>");
        std::process::exit(1);
    }
    
    let source = fs::read_to_string(&args[1]).expect("Failed to read file");
    
    let mut parser = Parser::new();
    let language = tree_sitter_solidity::LANGUAGE.into();
    parser.set_language(&language).expect("Failed to set language");
    
    let tree = parser.parse(&source, None).expect("Failed to parse");
    
    println!("=== Parse Tree for {} ===\n", args[1]);
    print_tree(&tree.root_node(), &source, 0);
    
    println!("\n=== All Node Types ===\n");
    let mut types = std::collections::HashSet::new();
    collect_node_types(&tree.root_node(), &mut types);
}

fn print_tree(node: &tree_sitter::Node, source: &str, indent: usize) {
    let indent_str = "  ".repeat(indent);
    
    if node.is_named() {
        let text_preview: String = node.utf8_text(source.as_bytes())
            .unwrap_or("")
            .chars()
            .take(50)
            .collect();
        let text_preview = text_preview.replace('\n', "\\n");
        
        println!(
            "{}({}) [{}-{}] \"{}{}\"",
            indent_str,
            node.kind(),
            node.start_position().row,
            node.end_position().row,
            text_preview,
            if text_preview.len() >= 50 { "..." } else { "" }
        );
    }
    
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.is_named() {
                print_tree(&child, source, indent + 1);
            }
        }
    }
}

fn collect_node_types(node: &tree_sitter::Node, types: &mut std::collections::HashSet<String>) {
    if node.is_named() && types.insert(node.kind().to_string()) {
        println!("  {}", node.kind());
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            collect_node_types(&child, types);
        }
    }
}
