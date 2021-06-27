## Door Protocol for Applications
*Also Known As:*<br/>
[![HOW CRAB SHAKE HAND](artwork/aj-pixels-crab-shake-hand.jpg)][2]


### Outline

* The application opens an illumos door for the PortunusD server.
* Each network request is delivered to the application in a single `door_call`.
* Each response must fit into a single `door_return` buffer (1024KB max).
* The PortunusD server will not share descriptors with the application.
* There is no explicit error handling, but applications may choose to respond
   with a zero-length payload.


### History & Versioning

To see previous protocol specifications, either run `git log -- etc/DPA.md`
inside this repository, or visit [History for etc/DPA.md][1] on GitHub. At this
time, the DPA is not stable, so every revision should be considered a breaking
change.

Once the DPA and PortunusD itself are stable, users will be able to expect that
breaking changes to the protocol will only be introduced in new major versions
of PortunusD. Minor- and patch-level updates to the server will not break
existing applications.

[1]: https://github.com/robertdfrench/portunusd/commits/trunk/etc/DPA.md
[2]: https://www.youtube.com/watch?v=rU2X_N7M6-E
