use conn_test::utils::Url;

#[test]
fn should_verify_url() {
    let input = compact_str::CompactString::const_new("tcp://test.com/");
    let url = Url::parse(input).expect("parse url");
    assert_eq!(url.schema(), Some("tcp"));
    assert_eq!(url.username(), None);
    assert_eq!(url.password(), None);
    assert_eq!(url.host(), "test.com");
    assert_eq!(url.path(), "/");
    assert_eq!(url.port(), None);
    assert_eq!(url.to_string(), "tcp://test.com/");

    let input = compact_str::CompactString::const_new("https://user@test.com/echo");
    let url = Url::parse(input).expect("parse url");
    assert_eq!(url.schema(), Some("https"));
    assert_eq!(url.username(), Some("user"));
    assert_eq!(url.password(), None);
    assert_eq!(url.host(), "test.com");
    assert_eq!(url.path(), "/echo");
    assert_eq!(url.to_string(), "https://user@test.com/echo");

    let input = compact_str::CompactString::const_new("https://user:@test.com/echo");
    let url = Url::parse(input).expect("parse url");
    assert_eq!(url.schema(), Some("https"));
    assert_eq!(url.username(), Some("user"));
    assert_eq!(url.password(), Some(""));
    assert_eq!(url.host(), "test.com");
    assert_eq!(url.path(), "/echo");
    assert_eq!(url.to_string(), "https://user:@test.com/echo");

    let input = compact_str::CompactString::const_new("https://user:pass@test.com/echo");
    let url = Url::parse(input).expect("parse url");
    assert_eq!(url.schema(), Some("https"));
    assert_eq!(url.username(), Some("user"));
    assert_eq!(url.password(), Some("pass"));
    assert_eq!(url.host_port(), "test.com");
    assert_eq!(url.host(), "test.com");
    assert_eq!(url.path(), "/echo");
    assert_eq!(url.to_string(), "https://user:****@test.com/echo");

    let input = compact_str::CompactString::const_new("https://user:pass@test.com:8080/echo");
    let url = Url::parse(input).expect("parse url");
    assert_eq!(url.schema(), Some("https"));
    assert_eq!(url.username(), Some("user"));
    assert_eq!(url.password(), Some("pass"));
    assert_eq!(url.host(), "test.com");
    assert_eq!(url.host_port(), "test.com:8080");
    assert_eq!(url.port(), Some("8080"));
    assert_eq!(url.port_u16(), Some(8080));
    assert_eq!(url.path(), "/echo");
    assert_eq!(url.to_string(), "https://user:****@test.com:8080/echo");
}
