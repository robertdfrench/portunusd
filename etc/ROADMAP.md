## Future Work
[Milestones](https://github.com/robertdfrench/portunusd/milestones)
&VerticalSeparator;
[Changelog](https://github.com/robertdfrench/portunusd/blob/trunk/CHANGELOG.md)
&VerticalSeparator;
[Releases](https://github.com/robertdfrench/portunusd/releases)


*This roadmap attempts to show a rough outline of the development priorities for
PortunusD. Each item in the outline corresponds to one Milestone. No dates are
specified, but the idea is to tackle them more-or-less chronologically.*


### [Example DNS Server]
To be included with the default `portunusd` release as a demo app, a caching DNS
server which forwards all requests to 8.8.8.8. It does not need configurability,
as it is only there to demonstrate `portunusd`'s UDP handling capabilities.


### [HTTP Routing]
So-called "Layer 7" routing. Instead of specifying `tcp` or `udp` as the
incoming protocol, users can instruct `portunusd` to listen to "http" on a given
port, and forward requests to different doors based on the URI path.


### [TLS Termination]
Handle TLS connections and certificates so that the applications do not have to.
This allows application developers to focus on request handling logic. This will
either require `portunusd` to know the domain name for each TLS request it
forwards, or it will require there to be only one domain associated with a given
portunusd instance. 


### [Load Balancing]
When resources are constrained, forward requests to separate `portunusd` hosts.
Each host will need to be configured to trust the other; backup hosts should
know to expect forwarded requests from primary hosts, and each host will need to
be able to verify the identity of the other. Serious skepticism should be given
to any deviation from `relayd`'s procedures for this.


### [Dogfood the Website]
The www.portunusd.net website should be served by PortunusD in conjunction with
the static-files-over-http example application.


[Example DNS server]: https://github.com/robertdfrench/portunusd/milestone/9
[HTTP Routing]: https://github.com/robertdfrench/portunusd/milestone/10
[TLS Termination]: https://github.com/robertdfrench/portunusd/milestone/4
[Load Balacing]: https://github.com/robertdfrench/portunusd/milestone/5
[Dogfood the Website]: https://github.com/robertdfrench/portunusd/milestones/8
