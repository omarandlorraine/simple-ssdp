use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use crate::MulticastAddr;

pub(crate) fn join_socket(address: &MulticastAddr, socket: Arc<UdpSocket>) -> Result<(), Box<dyn std::error::Error>> {
    // Join the multicast group
    
    if address == &MulticastAddr::Loopback {
        return Ok(());
    }
    
    if address.is_v4() {
        socket.join_multicast_v4(
            address.get_v4().expect("We just checked that it's v4"),
            Ipv4Addr::UNSPECIFIED,
        )?;
    } else {
        socket
            .join_multicast_v6(&address.get_v6().expect("We just checked that it's v6"), 0)?;
    }
    
    Ok(())
}