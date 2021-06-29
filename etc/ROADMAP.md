## Future Work
[Milestones](https://github.com/robertdfrench/portunusd/milestones)
&VerticalSeparator;
[Changelog](https://github.com/robertdfrench/portunusd/blob/trunk/CHANGELOG.md)
&VerticalSeparator;
[Releases](https://github.com/robertdfrench/portunusd/releases)


*This roadmap attempts to show a rough outline of the development priorities for
PortunusD. Each item in the outline corresponds to one Milestone. No dates are
specified, but the idea is to tackle them more-or-less chronologically.*


### UDP

#### [Example DNS Server]
To be included with the default `portunusd` release as a demo app, a caching DNS
server which forwards all requests to 8.8.8.8. It does not need configurability,
as it is only there to demonstrate `portunusd`'s UDP handling capabilities.

#### [Dogfood Nameservers]
Instead of relying on an established, competent DNS provider like
[Gandi](https://www.gandi.net), set up a pair of `portunusd` instances running
the example DNS server and list them as the authoritative nameservers for
`portunusd.net`.


### HTTP

#### [HTTP Routing]
So-called "Layer 7" routing. Instead of specifying `tcp` or `udp` as the
incoming protocol, users can instruct `portunusd` to listen to "http" on a given
port, and forward requests to different doors based on the URI path.

#### [Dogfood the Website]
The www.portunusd.net website should be served by PortunusD in conjunction with
the static-files-over-http example application.


### TLS

#### [TLS Termination]
Handle TLS connections and certificates so that the applications do not have to.
This allows application developers to focus on request handling logic. This will
either require `portunusd` to know the domain name for each TLS request it
forwards, or it will require there to be only one domain associated with a given
portunusd instance. 

#### [Dogfood CI]
Handle GitHub webhooks using a PortunusD web app. Specifically, when pull
requests are created or updated, have the CI app clone the repo, check out the
commit mentioned in the pull request, and run `cargo test --verbose` in a
disposable zone.

#### [Load Balancing]
When resources are constrained, forward requests to separate `portunusd` hosts.
Each host will need to be configured to trust the other; backup hosts should
know to expect forwarded requests from primary hosts, and each host will need to
be able to verify the identity of the other. Serious skepticism should be given
to any deviation from `relayd`'s procedures for this.


[Example DNS server]: https://github.com/robertdfrench/portunusd/milestone/9
[Dogfood Nameservers]: https://github.com/robertdfrench/portunusd/milestone/6
[HTTP Routing]: https://github.com/robertdfrench/portunusd/milestone/10
[Dogfood the Website]: https://github.com/robertdfrench/portunusd/milestone/8
[TLS Termination]: https://github.com/robertdfrench/portunusd/milestone/4
[Dogfood CI]: https://github.com/robertdfrench/portunusd/milestone/11
[Load Balancing]: https://github.com/robertdfrench/portunusd/milestone/5
