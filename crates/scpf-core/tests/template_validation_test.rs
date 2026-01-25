/// Integration test to validate SCPF templates against real vulnerable contract patterns
/// Tests patterns extracted from manual analysis of 20 high-risk contracts

use scpf_core::Scanner;
use scpf_types::Template;
use std::path::PathBuf;

const TEST_CONTRACT_PATH: &str = "../../sol/test_vulnerable_patterns.sol";

fn load_template(path: &str) -> Vec<Template> {
    let content = std::fs::read_to_string(path).expect("Failed to read template");
    let template: Template = serde_yaml::from_str(&content).expect("Failed to parse template");
    println!("Loaded template: id={}, patterns={}", template.id, template.patterns.len());
    for (i, p) in template.patterns.iter().enumerate() {
        println!("  Pattern {}: id={}, pattern={}", i, p.id, p.pattern);
    }
    vec![template]
}

#[test]
fn test_weak_randomness_detection() {
    let templates = load_template("../../templates/weak_randomness.yaml");
    let contract_code = std::fs::read_to_string(TEST_CONTRACT_PATH).expect("Failed to read contract");
    
    let mut scanner = Scanner::new(templates).expect("Failed to create scanner");
    let results = scanner.scan(&contract_code, PathBuf::from("test_vulnerable_patterns.sol")).expect("Scan failed");
    
    println!("Total findings: {}", results.len());
    for m in &results {
        println!("  - Pattern: {}, Line: {}, Text: {}", m.pattern_id, m.line_number, m.matched_text);
    }
    
    let blockhash_findings: Vec<_> = results.iter()
        .filter(|m| m.pattern_id.contains("blockhash"))
        .collect();
    
    assert!(!blockhash_findings.is_empty(), 
        "FAIL: weak_randomness.yaml did not detect blockhash usage in OracleRNG_Vulnerable");
    
    println!("✓ PASS: Detected weak randomness (blockhash)");
}

#[test]
fn test_arbitrary_call_detection() {
    let templates = load_template("../../templates/unchecked_return_value.yaml");
    let contract_code = std::fs::read_to_string(TEST_CONTRACT_PATH).expect("Failed to read contract");
    
    let mut scanner = Scanner::new(templates).expect("Failed to create scanner");
    let results = scanner.scan(&contract_code, PathBuf::from("test_vulnerable_patterns.sol")).expect("Scan failed");
    
    let arbitrary_call_findings: Vec<_> = results.iter()
        .filter(|m| m.pattern_id.contains("arbitrary-call") || m.pattern_id.contains("call"))
        .collect();
    
    assert!(!arbitrary_call_findings.is_empty(), 
        "FAIL: unchecked_return_value.yaml did not detect arbitrary call in MoonCatsStrategyV2_Vulnerable");
    
    println!("✓ PASS: Detected arbitrary external call with data");
}

#[test]
fn test_reentrancy_erc20_detection() {
    let templates = load_template("../../templates/reentrancy.yaml");
    let contract_code = std::fs::read_to_string(TEST_CONTRACT_PATH).expect("Failed to read contract");
    
    let mut scanner = Scanner::new(templates).expect("Failed to create scanner");
    let results = scanner.scan(&contract_code, PathBuf::from("test_vulnerable_patterns.sol")).expect("Scan failed");
    
    println!("Total findings: {}", results.len());
    for m in &results {
        println!("  - Pattern: {}, Line: {}, Text: {}", m.pattern_id, m.line_number, m.matched_text);
    }
    
    let transferfrom_findings: Vec<_> = results.iter()
        .filter(|m| {
            println!("    Checking: pattern_id={}, filtered={}", m.pattern_id, m.filtered);
            m.pattern_id.contains("transferfrom") || m.pattern_id.contains("erc20") || m.pattern_id.contains("before-state")
        })
        .collect();
    
    assert!(!transferfrom_findings.is_empty(), 
        "FAIL: reentrancy.yaml did not detect transferFrom before state change in BondingCurve_Vulnerable");
    
    println!("✓ PASS: Detected ERC20 transferFrom reentrancy");
}

#[test]
fn test_signature_replay_detection() {
    let templates = load_template("../../templates/signature_unchecked.yaml");
    let contract_code = std::fs::read_to_string(TEST_CONTRACT_PATH).expect("Failed to read contract");
    
    let mut scanner = Scanner::new(templates).expect("Failed to create scanner");
    let results = scanner.scan(&contract_code, PathBuf::from("test_vulnerable_patterns.sol")).expect("Scan failed");
    
    let ecrecover_findings: Vec<_> = results.iter()
        .filter(|m| m.pattern_id.contains("ecrecover"))
        .collect();
    
    assert!(!ecrecover_findings.is_empty(), 
        "FAIL: signature_unchecked.yaml did not detect ecrecover in Channel_Vulnerable");
    
    println!("✓ PASS: Detected signature without replay protection");
}

#[test]
fn test_cross_chain_gas_grief_detection() {
    let templates = load_template("../../templates/cross_chain_gas_grief.yaml");
    let contract_code = std::fs::read_to_string(TEST_CONTRACT_PATH).expect("Failed to read contract");
    
    let mut scanner = Scanner::new(templates).expect("Failed to create scanner");
    let results = scanner.scan(&contract_code, PathBuf::from("test_vulnerable_patterns.sol")).expect("Scan failed");
    
    let call_findings: Vec<_> = results.iter()
        .filter(|m| m.pattern_id.contains("call") || m.pattern_id.contains("eth-transfer"))
        .collect();
    
    let message_findings: Vec<_> = results.iter()
        .filter(|m| m.pattern_id.contains("message") || m.pattern_id.contains("sendMessage"))
        .collect();
    
    assert!(!call_findings.is_empty(), 
        "FAIL: cross_chain_gas_grief.yaml did not detect .call{{value:}} in TransferRegistry_Vulnerable");
    
    assert!(!message_findings.is_empty(), 
        "FAIL: cross_chain_gas_grief.yaml did not detect sendMessage in TransferRegistry_Vulnerable");
    
    println!("✓ PASS: Detected cross-chain gas griefing pattern");
}

#[test]
fn test_delegatecall_detection() {
    let templates = load_template("../../templates/delegatecall_user_input.yaml");
    let contract_code = std::fs::read_to_string(TEST_CONTRACT_PATH).expect("Failed to read contract");
    
    let mut scanner = Scanner::new(templates).expect("Failed to create scanner");
    let results = scanner.scan(&contract_code, PathBuf::from("test_vulnerable_patterns.sol")).expect("Scan failed");
    
    println!("Total findings: {}", results.len());
    for m in &results {
        println!("  - Pattern: {}, Line: {}, Text: {}", m.pattern_id, m.line_number, m.matched_text);
    }
    
    let delegatecall_findings: Vec<_> = results.iter()
        .filter(|m| m.pattern_id.contains("delegatecall"))
        .collect();
    
    assert!(!delegatecall_findings.is_empty(), 
        "FAIL: delegatecall_user_input.yaml did not detect delegatecall in IdentityRegistry_Vulnerable");
    
    println!("✓ PASS: Detected delegatecall to user-controlled address");
}

#[test]
fn test_callback_reentrancy_detection() {
    let templates = load_template("../../templates/reentrancy_callback.yaml");
    let contract_code = std::fs::read_to_string(TEST_CONTRACT_PATH).expect("Failed to read contract");
    
    let mut scanner = Scanner::new(templates).expect("Failed to create scanner");
    let results = scanner.scan(&contract_code, PathBuf::from("test_vulnerable_patterns.sol")).expect("Scan failed");
    
    let callback_findings: Vec<_> = results.iter()
        .filter(|m| m.pattern_id.contains("erc1155") || m.pattern_id.contains("callback"))
        .collect();
    
    assert!(!callback_findings.is_empty(), 
        "FAIL: reentrancy_callback.yaml did not detect onERC1155Received in AlpacaFarm_Vulnerable");
    
    println!("✓ PASS: Detected ERC1155 callback reentrancy");
}
