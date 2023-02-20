# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# Copyright 2021 Robert D. French


# The default task in our makefile, "help", will print a small table of
# user-invocable targets and corresponding descriptions. This convention is
# borrowed from Rails' "rake" command, and makes it easy to use a Makefile as
# the universal point-of-entry for development tasks.
#
help: #: Build this help menu from Makefile target comments (default)
	@echo "USAGE:\n"
	@awk -F ':' 'NF >= 3 { OFS="#"; print "- make "$$1,$$3 }' Makefile \
		| sort \
		| tr '#' '\t'


# This task generates documentation and opens a web browser, so it should only
# run on the developer's workstation.
#
docs: #: Build documentation from Rust source files
	cargo doc

docweb: docs #: Serve documentation on the local network
	cd target/doc && python3 -m http.server 8000

test: #: Run unit tests 
	cargo test

clean: #: Clean up so we can rebuild from scratch
	cargo clean
