use fancy_regex::Regex;

#[test]
fn test_fancy_regex_transferfrom() {
    let pattern = r"(?:baseToken|token|IERC20|ERC20)\s*\.\s*transferFrom\s*\(";
    let text = "require(baseToken.transferFrom(msg.sender, address(this), baseIn));";

    let regex = Regex::new(pattern).expect("Failed to compile regex");
    let result = regex.find(text).expect("Regex find failed");

    assert!(result.is_some(), "Pattern should match");
    println!("Match found: {:?}", result.unwrap().as_str());
}

#[test]
fn test_fancy_regex_delegatecall() {
    let pattern = r"(?:\.|\s)delegatecall\s*\(";
    let text = "let success := delegatecall(sub(gas(), 10000), logic, 0x0, calldatasize(), 0, 0)";

    let regex = Regex::new(pattern).expect("Failed to compile regex");
    let result = regex.find(text).expect("Regex find failed");

    assert!(result.is_some(), "Pattern should match");
    println!("Match found: {:?}", result.unwrap().as_str());
}
