use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use log::debug;
use log::trace;
use tokio::net::UdpSocket;
use tokio::time::timeout;

use crate::http_helper::generate_ssdp_discover;
use crate::service::ServiceDescription;
use crate::socket_helper::join_socket;
use crate::MulticastAddr;
use crate::SSDP_PORT;

/// The SSDP Client
pub struct Client {
    /// List of Services found by the Client
    ///
    /// | USN URI          | Service Type URI | Expiration | Location          |
    /// |------------------|------------------|------------|-------------------|
    /// | upnp:uuid:k91... | upnp:clockradio  | 3 days     | http://foo.com/cr |
    /// | uuid:x7z...      | ms:wince         | 1 week     | http://msce/win   |
    services: Arc<Mutex<Vec<ServiceDescription>>>,

    /// Timeout - used to wait for incoming answers
    timeout: Duration,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            services: Arc::new(Mutex::new(vec![])),
            timeout: Duration::from_secs(5),
        }
    }
}

impl Client {
    // TODO there could be a blocking channel which also receives the NOTIFY and BYEBYE calls
    /// Discover SSDP Services
    /// - `identifier`: The unique Identifier for this Client e.g. `uuid:83760048-2d32-4e48-854f-f63a8fa9fd09`
    /// - `address`: In which scope do you want to scan?
    /// - `search`: `ssdp:all` to find all SSDP Services or custom Service Types to look for
    pub async fn discover(
        &self,
        identifier: String,
        address: MulticastAddr,
        search: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create a UDP socket
        let local_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let socket = Arc::new(UdpSocket::bind(local_addr).await?);
        let multicast_addr: SocketAddr = SocketAddr::new(address.get_ip(), SSDP_PORT);

        join_socket(&address, socket.clone())?;

        let discover_message = generate_ssdp_discover(identifier, search, &address);

        // Multicast search request
        socket
            .send_to(discover_message.as_bytes(), &multicast_addr)
            .await?;

        // Create a buffer to store the received data
        let mut buf = vec![0; 1024];

        // Listen for service replies
        // Define a timeout duration for listening for responses
        let start = tokio::time::Instant::now();

        // Listen for service replies until timeout
        while start.elapsed() < self.timeout {
            match timeout(
                self.timeout - start.elapsed(),
                socket.recv_from(&mut buf),
            )
            .await
            {
                Ok(Ok((len, addr))) => {
                    let response_message = String::from_utf8_lossy(&buf[..len]).to_string();
                    trace!(
                        "Received {} bytes from {}: {:#?}",
                        len,
                        addr,
                        response_message
                    );

                    let mut headers = [httparse::EMPTY_HEADER; 64];
                    let mut resp = httparse::Response::new(&mut headers);

                    if resp.parse(&buf[..len]).is_err() {
                        trace!("Could not parse request to HTTP");
                        continue;
                    }

                    if resp.code.is_none() || resp.code.is_some_and(|code| 200 != code) {
                        trace!("Response Code is not 200");
                        continue;
                    }

                    // TODO the response must be in exact order right now
                    let usn = resp.headers.get(4);
                    if usn.is_none() || usn.is_some_and(|usn| usn.name != "USN") {
                        trace!("USN header is not present");
                        continue;
                    }

                    let st = resp.headers.get(3);
                    if st.is_none() || st.is_some_and(|st| st.name != "ST") {
                        trace!("ST header is not present");
                        continue;
                    }

                    let al = resp.headers.get(5);
                    if al.is_none() || al.is_some_and(|al| al.name != "AL") {
                        trace!("ST header is not present");
                        continue;
                    }

                    let new_service = ServiceDescription {
                        usn_uri: String::from_utf8_lossy(usn.expect("Error with USN-Header").value)
                            .into(),
                        service_type_uri: String::from_utf8_lossy(
                            st.expect("Error with ST-Header").value,
                        )
                        .into(),
                        expiration: 100, // TODO needs to get parsed from max-age=<someint>
                        location: String::from_utf8_lossy(al.expect("Error with AL-Header").value)
                            .into(), // TODO is: <some:service><http://foo/bar> but should be http://foo/bar
                    };

                    let mut services_guard = self.services.lock().unwrap();
                    let mut found = false;

                    for service in services_guard.iter_mut() {
                        if service.usn_uri == new_service.usn_uri {
                            *service = new_service.clone();
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        services_guard.push(new_service);
                    }

                    buf.clear();
                    buf.resize(1024, 0);
                }
                Ok(Err(e)) => {
                    trace!("Error receiving response: {}", e);
                }
                Err(_) => {
                    break; // Timeout reached
                }
            }
        }

        debug!("Services found: {:#?}", self.services);

        Ok(())
    }
    
    /// Retrieve a list of all Services that answered to our multicast call
    pub fn get_services(&self) -> Vec<ServiceDescription> {
        self.services.lock().unwrap().clone()
    }

    /// Changes the timeout
    pub fn set_timeout(&mut self, timeout: Duration) -> &Self {
        self.timeout = timeout;

        self
    }
}
