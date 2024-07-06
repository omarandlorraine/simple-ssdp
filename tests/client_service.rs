use std::io::Error;
use std::time::Duration;
use ssdp::client::Client;
use ssdp::service::Service;
use ssdp::service::ServiceDescription;
use ssdp::MulticastAddr;

#[tokio::test]
/// Spinning up a client and a service on localhost to test the connection
/// 
/// This takes a while to reach the client's timeout.
async fn test_client_and_service_communication() {
    async fn listen() -> Result<(), Error> {
        let desc = ServiceDescription {
            usn_uri: "uuid:some-service-uuid".to_string(),
            service_type_uri: "some:special:service".to_string(),
            expiration: 100,
            location: "https://foo/bar".to_string(),
        };

        let service = Service::new(desc);
        let _ = service.listen(MulticastAddr::Loopback).await;

        Ok(())
    }

    let thread_listen = tokio::spawn(async move { listen().await });

    let mut client = Client::default();
    client
        .set_timeout(Duration::from_millis(500));

    client
        .discover("uuid:some-client-uuid".to_string(),
                  MulticastAddr::Loopback,
                  "some:special:service".to_string())
        .await.unwrap();

    thread_listen.abort();
    
    let expected: Vec<ServiceDescription> = vec![ServiceDescription{
        usn_uri: "uuid:some-service-uuid".to_string(),
        service_type_uri: "some:special:service".to_string(),
        expiration: 100,
        location: "<some:special:service><https://foo/bar>".to_string(),
    }];
    
    assert_eq!(expected, client.get_services());
}
