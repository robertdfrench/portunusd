## illumos APIs not covered in libc crate
*or, at least, the subset we need for portunusd*

These modules redefine several types from illumos headers, mostly to do with
doors. These types have not yet been incorporated into the [libc] crate.

Each type is defined in a module that corresponds to the name of the header from
which it originates, making it easier to validate the fidelity of this
re-implementation.

[libc]: https://crates.io/crates/libc
