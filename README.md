# Astrea
Astrea is a lightweight, highly-configurable and extensible load balancer. Astrea is currently in an alpha-state, but the code on master should function as expected. Right now it can perform round-robin load balancing on plain TCP, HTTP, and HTTPS.

## Configuration
```
host: 127.0.0.1 # Required
port: 8080 # Binds to 80 by default

# Define your endpoints below. For HTTP you can input regular URLs. TCP will
# take endpoints of the form ip:port.
endpoints: 
  - "https://example.com"
  - "https://google.com"

# Choose endpoint selection strategy. Right now only round robin is implemented.
endpoint-selector: round robin

# Protocol. Choices are http or tcp.
protocol: http

# Optional section for setting up TLS
https: 
  # Path to PKCS#12 identity file containing TLS certificate
  identity-file: astrea.p12 
  # Password of file, if any
  password: astrea
  # Port of TLS server, uses 443 by default
  port: 8081
```

## Goals
 - ~~Plain TCP load balancing~~
 - ~~HTTP load balancing~~
 - ~~HTTPS load balancing and termination~~
 - ~~Internal refactoring to make new endpoint selection algorithms easier to add~~
 - Better error handling
 - HTTP/2.0 support
 - Better usability ergonomics
 - Performance tuning
 - Documentation
