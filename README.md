# OpenSyndicationProtocol
A server -> server RPC protocol for syndicating data, implemented in rust.

# Completion State
This project is a work in progress:
- [x] Protocol Transport
- [x] Handshake Frame
  - [x] DNS-based RSA-signed challenge sequence
- [ ] Data Types
  - [ ] Node data type registry
  - [ ] Protocol frame for communicating data capabilities with other servers
  - [ ] Universal Data -> buffer serialization/deserialization framework for consumers
- [ ] Client -> server communication for devs that wish to build client -> server architecture with all syndicated data availible to the client
  - todo: elaborate this point