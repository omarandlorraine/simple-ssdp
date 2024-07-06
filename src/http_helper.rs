use crate::service::ServiceDescription;
use crate::MulticastAddr;
use crate::SSDP_PORT;

#[allow(dead_code)]
// TODO remove allow(dead_code) when this is implemented
/// Generates ssdp:alive
///
/// This should be multicasted when a [crate::service::Service] is starting.
pub(crate) fn generate_ssdp_alive(
    service_description: &ServiceDescription,
    ssdp_multicast_addr: &MulticastAddr,
) -> String {
    format!(
        "NOTIFY * HTTP/1.1\r\nHost: {}:{}\r\nNT: {}\r\nNTS: ssdp:alive\r\nUSN: {}\r\nAL: <{}><{}>\r\nCache-Control: max-age = {}\r\n\r\n",
        ssdp_multicast_addr.get_ip(),
        SSDP_PORT,
        service_description.service_type_uri,
        service_description.usn_uri,
        service_description.usn_uri,
        service_description.location,
        service_description.expiration,
    )
}

#[allow(dead_code)]
// TODO remove allow(dead_code) when this is implemented
/// Generates ssdp:byebye
///
/// This should be multicasted when a [crate::service::Service] is stopping.
pub(crate) fn generate_ssdp_byebye(
    service_description: &ServiceDescription,
    ssdp_multicast_addr: &MulticastAddr,
) -> String {
    format!(
        "NOTIFY * HTTP/1.1\r\nHost: {}:{}\r\nNT: {}\r\nNTS: ssdp:byebye\r\nUSN: {}\r\n\r\n",
        ssdp_multicast_addr.get_ip(),
        SSDP_PORT,
        service_description.service_type_uri,
        service_description.usn_uri,
    )
}

/// Answer to a `M-SEARCH` request
///
/// - `service_description` - The descriptive object of the [crate::service::Service]
/// - `s` - The unique identifier of the requesting [crate::client::Client]
pub(crate) fn generate_ssdp_discover_answer(
    service_description: &ServiceDescription,
    s: String
) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nS: {}\r\nExt: \r\nCache-Control: no-cache=\"Ext\", max-age={}\r\nST: {}\r\nUSN: {}\r\nAL: <{}><{}>\r\n\r\n",
        s,
        service_description.expiration,
        service_description.service_type_uri,
        service_description.usn_uri,
        service_description.service_type_uri,
        service_description.location,
    )
}

// Helper for the Client
/// Generates a `M-SEARCH` request to find [crate::service::Service] within the given multicast network
///
/// - `s` - The unique identifier of this [crate::client::Client], mostly an uuid e.g. `uuid:83760048-2d32-4e48-854f-f63a8fa9fd09`
/// - `st` - A name to search for, can be `ssdp:all` to find all services or a more specific phrase like `my:service`
/// - `ssdp_multicast_addr` - The multicast network to announce this search request
pub(crate) fn generate_ssdp_discover(
    s: String,
    st: String,
    ssdp_multicast_addr: &MulticastAddr,
) -> String {
    // TODO what even is this MX header???
    // TODO error in RFC? MAN is the only quoted parameter
    format!(
        "M-SEARCH * HTTP/1.1\r\nS: {}\r\nHost: {}:{}\r\nMAN: \"ssdp:discover\"\r\nST: {}\r\nMX: {}\r\n\r\n",
        s,
        ssdp_multicast_addr.get_ip(),
        SSDP_PORT,
        st,
        1,
    )
}
