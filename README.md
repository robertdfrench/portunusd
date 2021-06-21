# PortunusD
[Crate](https://crates.io/crates/portunusd) &VerticalSeparator;
[Docs](https://docs.rs/portunusd)           &VerticalSeparator;
[Tweets](https://twitter.com/portunusd)

`portunusd` is a network application server inspired by OpenBSD's [`relayd`][1]
and heirloom UNIX [`inetd`][2].  It listens for an incoming network connection,
forwarding the incoming data over an [illumos door][3] to the intended
application, and returning the response in a similar manner.  `portunusd` maps
each connected port to a door on the filesystem provided by the target
application.

![Startup and Request Handling](etc/diagrams/startup-and-request-handling.png)

The main goal of `portunusd` is to facilitate the scaling of single-threaded
applications. Under the `inetd` model, a new process is created to handle every
request. By leveraging doors, `portunusd` can create a new thread in the
application process only when a new highwater mark of concurrency has been
reached; otherwise, existing threads will be re-used to handle subsequent
requests.

### Problem Statement
We want our network-facing applications to scale according to user demand. We
want to minimize the resource cost of our applications when they are idle, and
we want to keep our costs linear in terms of demand. We want to
minimize the degree to which the application developer is responsible for
resource management, and we want to retain (so far as possible) the familiar
development environment of UNIX command line tools.

Picking on Rails as an example, a single-threaded Ruby on Rails application can
handle one user request at a time. Multiple simultaneous requests cannot be
handled without multiple copies of the application resident in memory (on
separate Ruby interpreters). This model consumes a great deal of memory even
when there is little user demand, making it difficult for the host to run other
workloads. Much paging and gnashing of disk will ensue.

Environments such as Node.js deal with this problem by making asynchronicity
more transparent to the programmer. While it can be useful to embrace the
asynchronous nature of computers, it has also introduced changes to languages
that support it; this is not a mere change of syntax, but also a nontrivial
change to the mental model one uses to read, write, and understand programs.

At the other end of the spectrum, CGI applications require a unique process and
address space for each request. These applications can scale linearly with user
demand, including scaling down to zero memory / cpu usage when idle, but the
cost of invoking `execv(2)` for each request can hamper throughput.

The postmodern "Serverless" approach satisfies these criteria, but at the cost
of *abandoning the operating sytem*. It is a wildly unfamiliar approach to
developing software, and throws away many tools that could be used to observe
and debug the application at runtime.

### Thesis
Doors enable a new (old?) model of network application development wherein the
developers are responsible for maintaining and understanding a linear,
synchronous task, while the operating system + web server work together on the
scaling problem

* When an application is idle, only a single copy of is needed in memory.
* When a request enters the system, it can be handled by an existing thread.
* New threads are created only when a new peak of concurrency is reached.

These qualities allow us to address our problem statement by developing network
applications that feel like single-threaded UNIX command line tools, present a
minimal expense when idle, and scale linearly on a per-request granularity.

Of course, doors alone will not handle scaling across the boundary of a single
operating system instance, but a relayd-style collaboration with the firewall
could facilitate this, assuming copies of the application are available on
multiple hosts. This is where `portunusd` comes in.

### Acknowledgements
The social media preview image is by [Loudon dodd][4] - Own work, [CC BY-SA
3.0][5].

Many obscure illumos / Rust / Doors questions were answered by [@jasonbking][6].

<!-- References -->
[1]: https://github.com/openbsd/src/tree/master/usr.sbin/httpd
[2]: https://developer.ibm.com/technologies/linux/articles/au-spunix-inetd/
[3]: https://github.com/robertdfrench/revolving-door
[4]: https://commons.wikimedia.org/wiki/User:Loudon_dodd~commonswiki
[5]: https://creativecommons.org/licenses/by-sa/3.0
[6]: https://github.com/jasonbking
