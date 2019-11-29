# Astrea
Astrea is a lightweight, highly-configurable and extensible load balancer. Astrea is currently in an alpha-state, but the code on master should function as expected. Right now it can perform round-robin load balancing on plain TCP, HTTP, and HTTPS.

## Goals
 - ~~Plain TCP load balancing~~
 - ~~HTTP load balancing~~
 - ~~HTTPS load balancing and termination~~
 - Better error handling
 - HTTP/2.0 support
 - Internal refactoring to make new endpoint selection algorithms easier to add
 - Better usability ergonomics
 - Performance tuning
 - Documentation