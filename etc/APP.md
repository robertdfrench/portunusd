## Application Protocol for Portunus

1. The application opens an illumos door for the portunus server.
1. Each network request is delivered to the application in a single `door_call`.
1. Each response must fit into a single `door_return`.
1. The portunus server will not share descriptors with the application.
