## Portunus Doorway Protocol 2
*aka: PDP-2*

[![CRAB DO HUMAN A GREET][../artwork/aj-pixels-crab-shake-hand.jpg]][2]

1. The application opens an illumos door for the PortunusD server.
1. Each network request is delivered to the application in a single `door_call`.
1. Each response must fit into a single `door_return` buffer (1024KB max).
1. The PortunusD server will not share descriptors with the application.
1. There is no explicit error handling, but applications may choose to respond
   with a zero-length payload.


### History & Versioning

To see previous protocol specifications, either run `git log -- etc/PDP.md`
inside this repository, or visit [History for etc/PDP.md][1] on GitHub. At this
time, the PDP is not stable, so every revision should be considered a breaking
change.

Once the PDP and PortunusD itself are stable, users will be able to expect that
breaking changes to the protocol will only be introduced in new major versions
of PortunusD. Minor- and patch-level updates to the server will not break
existing applications.

[1]: https://github.com/robertdfrench/portunusd/commits/trunk/etc/PDP.md
[2]: https://www.youtube.com/watch?v=rU2X_N7M6-E
