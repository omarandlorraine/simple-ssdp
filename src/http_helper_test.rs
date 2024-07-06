use crate::http_helper::{generate_ssdp_alive, generate_ssdp_discover_answer};
use crate::http_helper::generate_ssdp_byebye;
use crate::http_helper::generate_ssdp_discover;
use crate::service::ServiceDescription;
use crate::MulticastAddr;
use crate::SSDP_PORT;

#[test]
fn test_alive() {
    let alive = generate_ssdp_alive(
        &ServiceDescription {
            usn_uri: "uuid:efef336d-fc25-4038-98f0-0217f6cc9e7a".to_string(),
            service_type_uri: "test:application".to_string(),
            expiration: 42,
            location: "https://foo/bar".to_string(),
        },
        &MulticastAddr::V4,
    );
    let buf = alive.as_bytes();

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    req.parse(buf).unwrap();

    assert_eq!("NOTIFY", req.method.unwrap());

    let host = req.headers.get(0);
    assert!(host.is_some());
    assert_eq!(
        format!("{}:{}", MulticastAddr::V4.get_ip(), SSDP_PORT,),
        String::from_utf8_lossy(host.unwrap().value)
    );

    let nt = req.headers.get(1);
    assert!(nt.is_some());
    assert_eq!(
        "test:application",
        String::from_utf8_lossy(nt.unwrap().value)
    );

    let nts = req.headers.get(2);
    assert!(nts.is_some());
    assert_eq!("ssdp:alive", String::from_utf8_lossy(nts.unwrap().value));

    let usn = req.headers.get(3);
    assert!(usn.is_some());
    assert_eq!(
        "uuid:efef336d-fc25-4038-98f0-0217f6cc9e7a",
        String::from_utf8_lossy(usn.unwrap().value)
    );

    let al = req.headers.get(4);
    assert!(al.is_some());
    assert_eq!(
        "<uuid:efef336d-fc25-4038-98f0-0217f6cc9e7a><https://foo/bar>",
        String::from_utf8_lossy(al.unwrap().value)
    );

    let cache_control = req.headers.get(5);
    assert!(cache_control.is_some());
    assert_eq!(
        "max-age = 42",
        String::from_utf8_lossy(cache_control.unwrap().value)
    );
}

#[test]
fn test_byebye() {
    let byebye = generate_ssdp_byebye(
        &ServiceDescription {
            usn_uri: "uuid:efef336d-fc25-4038-98f0-0217f6cc9e7a".to_string(),
            service_type_uri: "test:application".to_string(),
            expiration: 42,
            location: "https://foo/bar".to_string(),
        },
        &MulticastAddr::V4,
    );
    let buf = byebye.as_bytes();

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    req.parse(buf).unwrap();

    assert_eq!("NOTIFY", req.method.unwrap());

    let host = req.headers.get(0);
    assert!(host.is_some());
    assert_eq!(
        format!("{}:{}", MulticastAddr::V4.get_ip(), SSDP_PORT,),
        String::from_utf8_lossy(host.unwrap().value)
    );

    let nt = req.headers.get(1);
    assert!(nt.is_some());
    assert_eq!(
        "test:application",
        String::from_utf8_lossy(nt.unwrap().value)
    );

    let nts = req.headers.get(2);
    assert!(nts.is_some());
    assert_eq!("ssdp:byebye", String::from_utf8_lossy(nts.unwrap().value));

    let usn = req.headers.get(3);
    assert!(usn.is_some());
    assert_eq!(
        "uuid:efef336d-fc25-4038-98f0-0217f6cc9e7a",
        String::from_utf8_lossy(usn.unwrap().value)
    );
}

#[test]
fn test_discover_answer() {
    let discover_answer = generate_ssdp_discover_answer(
        &ServiceDescription{
            usn_uri: "uuid:83760048-2d32-4e48-854f-f63a8fa9fd09".to_string(),
            service_type_uri: "my:service".to_string(),
            expiration: 42,
            location: "https://foo/bar".to_string(),
        },
        "uuid:efef336d-fc25-4038-98f0-0217f6cc9e7a".to_string(),
    );
    let buf = discover_answer.as_bytes();

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut resp = httparse::Response::new(&mut headers);

    resp.parse(buf).unwrap();

    assert_eq!(200, resp.code.unwrap());

    let s = resp.headers.get(0);
    assert!(s.is_some());
    assert_eq!("uuid:efef336d-fc25-4038-98f0-0217f6cc9e7a", String::from_utf8_lossy(s.unwrap().value)
    );

    let ext = resp.headers.get(1);
    assert!(ext.is_some());
    assert_eq!("", String::from_utf8_lossy(ext.unwrap().value));

    let cache_control = resp.headers.get(2);
    assert!(cache_control.is_some());
    assert_eq!("no-cache=\"Ext\", max-age=42", String::from_utf8_lossy(cache_control.unwrap().value));


    let st = resp.headers.get(3);
    assert!(st.is_some());
    assert_eq!(
        "my:service",
        String::from_utf8_lossy(st.unwrap().value)
    );

    let usn = resp.headers.get(4);
    assert!(usn.is_some());
    assert_eq!(
        "uuid:83760048-2d32-4e48-854f-f63a8fa9fd09",
        String::from_utf8_lossy(usn.unwrap().value)
    );

    let al = resp.headers.get(5);
    assert!(al.is_some());
    assert_eq!(
        "<my:service><https://foo/bar>",
        String::from_utf8_lossy(al.unwrap().value)
    );
}

#[test]
fn test_discover() {
    let discover = generate_ssdp_discover(
        "uuid:efef336d-fc25-4038-98f0-0217f6cc9e7a".to_string(),
        "my:service".to_string(),
        &MulticastAddr::V4,
    );
    let buf = discover.as_bytes();

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    req.parse(buf).unwrap();

    assert_eq!("M-SEARCH", req.method.unwrap());

    let s = req.headers.get(0);
    assert!(s.is_some());
    assert_eq!("uuid:efef336d-fc25-4038-98f0-0217f6cc9e7a", String::from_utf8_lossy(s.unwrap().value)
    );

    let host = req.headers.get(1);
    assert!(host.is_some());
    assert_eq!(
        format!("{}:{}", MulticastAddr::V4.get_ip(), SSDP_PORT,),
        String::from_utf8_lossy(host.unwrap().value)
    );

    let man = req.headers.get(2);
    assert!(man.is_some());
    assert_eq!("\"ssdp:discover\"", String::from_utf8_lossy(man.unwrap().value));


    let st = req.headers.get(3);
    assert!(st.is_some());
    assert_eq!(
        "my:service",
        String::from_utf8_lossy(st.unwrap().value)
    );

    let mx = req.headers.get(4);
    assert!(mx.is_some());
    assert_eq!(
        "1",
        String::from_utf8_lossy(mx.unwrap().value)
    );
}
