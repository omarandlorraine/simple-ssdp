use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;

use log::debug;
use log::trace;
use tokio::net::UdpSocket;

use crate::http_helper::generate_ssdp_discover_answer;
use crate::socket_helper::join_socket;
use crate::MulticastAddr;
use crate::SSDP_PORT;

#[derive(Clone, PartialEq, Eq, Debug)]
/// This Describes the basic data exchanged between [Service] and [crate::client::Client]
///
/// Usages:
/// - It's used to create a new [Service]
/// - [crate::client::Client] holds a list of all it could find within the network
pub struct ServiceDescription {
    /// Unique Identifier, often a UUID
    ///     
    /// ```text
    /// upnp:uuid:83760048-2d32-4e48-854f-f63a8fa9fd09
    /// uuid:83760048-2d32-4e48-854f-f63a8fa9fd09
    /// ```
    pub usn_uri: String,

    /// Descriptive name, usually a [crate::client::Client] searches for this term
    ///
    /// ```text
    /// upnp:clockradio
    /// ms:wince
    /// my:app
    /// ```
    pub service_type_uri: String,

    /// Cache-Control max age
    ///
    /// The time this service usually expires and the [crate::client::Client] should search for again.
    ///
    /// According to [RFC 2616](https://www.ietf.org/rfc/rfc2616.txt) the maximum value is: 31536000
    pub expiration: u32,

    /// Location of the [Service], should be a valid URL
    ///
    /// ```text
    /// http://foo.com/bar
    /// https://myapp/service
    /// ```
    pub location: String,
}

/// The SSDP Service
///
/// Call [Service::new] with [ServiceDescription] to create a new [Service]
pub struct Service {
    service_description: ServiceDescription,
    // TODO we might want to hold a list of all Clients aswell
}

// TODO when starting Service send NOTIFY ssdp:alive to Multicast
// TODO when stopping Service send NOTIFY ssdp::byebye to Multicast
// TODO make this permanently running and accept Signals signaling to stop etc.

impl Service {
    /// Creates a new [Service]
    ///
    /// Requires a [ServiceDescription] to describe this Service
    pub fn new(service_description: ServiceDescription) -> Self {
        Service {
            service_description,
        }
    }

    /// Opens the listener
    ///
    /// This process is blocking so best to start it in its own thread
    pub async fn listen(&self, address: MulticastAddr) -> Result<(), Box<dyn std::error::Error>> {
        let local_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, SSDP_PORT));
        let socket = Arc::new(UdpSocket::bind(local_addr).await?);

        join_socket(&address, socket.clone())?;

        // Create a buffer to store the received data
        let mut buf = vec![0; 1024];

        debug!("Start listening for SSDP discovery messages...");

        // Listen for discovery requests and respond
        loop {
            let (len, addr) = socket.recv_from(&mut buf).await?;
            let copy = buf.clone();
            trace!(
                "Received {} bytes from {}: {:#?}",
                len,
                addr,
                String::from_utf8(copy).unwrap().replace('\0', "")
            );

            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut req = httparse::Request::new(&mut headers);

            if req.parse(&buf[..len]).is_err() {
                trace!("Could not parse request to HTTP");
                continue;
            }

            if req.method.is_none() || req.method.is_some_and(|method| method != "M-SEARCH") {
                trace!("Request is not M-SEARCH");
                continue;
            }

            // TODO right now this depends on the order of the header - as I only plan using client and service I don't really care
            let man = req.headers.get(2);
            if man.is_none()
                || man.is_some_and(|man| String::from_utf8_lossy(man.value) != "\"ssdp:discover\"")
            {
                trace!("Request uses wrong MAN header");
                continue;
            }

            let st = req.headers.get(3);
            if st.is_none()
                || st.is_some_and(|st| -> bool {
                    let st = String::from_utf8_lossy(st.value);

                    st != "ssdp:all" && st != self.service_description.service_type_uri
                })
            {
                trace!("ST header that's not interesting for us submitted");
                continue;
            }

            let s = req.headers.get(0);
            match s {
                None => {
                    trace!("S was not submitted");
                    continue;
                }
                Some(s) => {
                    let s = String::from_utf8_lossy(s.value);
                    let resp_msg =
                        generate_ssdp_discover_answer(&self.service_description, s.to_string());

                    socket.send_to(resp_msg.as_bytes(), &addr).await?;
                    trace!("Send SSDP response {:#?} to {}", resp_msg, addr);
                }
            }
            if s.is_none() {
                trace!("S was not submitted");
                continue;
            }
        }
    }
}
