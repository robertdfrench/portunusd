# C Proofs of Concept

```mermaid
sequenceDiagram
    Client->>+Proxy: Plz fork as me
    Proxy->>+Server: fork()
    Server-->>Proxy: Here is my door
    Proxy-->>-Client: Here is your door
    Client-->>Server: Invoke Door
```
