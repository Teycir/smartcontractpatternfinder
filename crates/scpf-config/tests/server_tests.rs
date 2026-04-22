use scpf_config::server_config_from_addr;

#[test]
fn server_origin_uses_http_for_ipv4_addresses() {
    let config = server_config_from_addr("127.0.0.1:32145");

    assert_eq!(config.addr, "127.0.0.1:32145");
    assert_eq!(config.origin, "http://127.0.0.1:32145");
}

#[test]
fn server_origin_wraps_ipv6_addresses_correctly() {
    let config = server_config_from_addr("[::1]:4444");

    assert_eq!(config.origin, "http://[::1]:4444");
}
