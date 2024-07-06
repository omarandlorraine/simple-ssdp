# SSDP
This is an implementation of the RFC draft from [SSDP](https://datatracker.ietf.org/doc/html/draft-cai-ssdp-v1-03).
However, it should be mostly compatible with the newer [UPnP](https://openconnectivity.org/upnp-specs/UPnP-arch-DeviceArchitecture-v2.0-20200417.pdf)

Feel free to contribute at any point.

# Planned features
 - [x] Send `M-SEARCH` request
 - [x] Answer `M-SEARCH` request
 - [x] Store a list of all services answering `M-SEARCH`
 - [ ] Send ALIVE when service comes up
 - [ ] Send BYEBYE when service goes down
 - [ ] Accept header in any order (right now only headers in a pre-defined order are working)

# Examples

### Service
To launch a Service that listens and answers to `M-SEARCH` requests do:

```rust
let desc = ServiceDescription {
    usn_uri: "uuid:83760048-2d32-4e48-854f-f63a8fa9fd09".to_string(), // TODO get from db
    service_type_uri: "AccessTime:Multicast".to_string(),
    expiration: 100,
    location: "https://127.0.0.1/api/v1/adopt".to_string(), // TODO get a servername from conf
};

let service = Service::new(desc);
service.listen(MulticastAddr::Loopback).await;
```

### Client
A client sends a `M-SEARCH` request and stores a list of all answering services.

```rust
let log = LogConfig{
    syslog_server: None,
    syslog_port: None,
    syslog_protocol: None,
    log_level: Some("trace".to_string()),
};

log.initialize_logger();

let client = Client::default();

client
    .discover("uuid:83760048-2d32-4e48-854f-f63a8fa9fd09".to_string(), MulticastAddr::Loopback, "AccessTime:Multicast".to_string())
    .await?;
```

Now you can fetch a `Vec<ServiceDescription>` with all answering services using `client.get_services()`

# License
To be fair this is just a setup I need for another project so feel free to do whatever you like with this. So feel free to choose between:
 - [Apache](APACHE-LICENSE.txt) License, Version 2.0 [apache.org](http://www.apache.org/licenses/LICENSE-2.0)
 - [MIT](MIT-LICENSE.txt) License [opensource.org](http://opensource.org/licenses/MIT)

# Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
