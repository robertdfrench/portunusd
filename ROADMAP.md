## Portunus Feature Roadmap

* Distributed DNS Server
 * Forwarding Port traffic to Door Applications
 * Loading balancing with `portunusd` instances on other hosts
* Distributed Web Server
 * Serving static files from disk
 * Routing HTTP to different Door Apps based on URI Path
* Terminating TLS on behalf of Door Applications (https://github.com/ctz/rustls)