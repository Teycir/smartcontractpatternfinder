use scpf_config::load_api_keys_from_lookup;
use scpf_types::Chain;
use std::collections::{BTreeSet, HashMap};

#[test]
fn etherscan_keys_are_shared_across_supported_chains() {
    let values = HashMap::from([
        ("ETHERSCAN_API_KEY", "key-one".to_string()),
        ("ETHERSCAN_API_KEY_2", "key-two".to_string()),
    ]);

    let config = load_api_keys_from_lookup(|name| values.get(name).cloned());

    let expected: BTreeSet<_> = ["key-one", "key-two"].into_iter().collect();

    for chain in [Chain::Ethereum, Chain::Polygon, Chain::Arbitrum] {
        let actual: BTreeSet<_> = config
            .get(chain)
            .unwrap()
            .iter()
            .map(String::as_str)
            .collect();
        assert_eq!(actual, expected);
    }
}
