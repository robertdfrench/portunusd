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
	@awk -F ':' 'NF >= 3 { OFS="#"; print "-",$$1,$$3 }' $(MAKEFILE_LIST) \
		| sort | column -s "#" -t

# Remote development can be a pain. The "smartos" macro makes it possible to run
# commands either from a smartos host or from a workstation which has ssh
# access to a smartos host. If a task wraps all its commands in the smartos
# macro, it can be invoked from either a smartos host or a remote workstation.
ifeq ($(findstring joyent,$(shell uname -v)),joyent)
smartos=$(1)
else
smartos=@$(MAKE) sync \
	&& ssh -t "${SMARTOS_HOST}" \
		bash -c 'true; set -e; cd portunus; set -x; $(1)'
endif

sync: #: Push latest code to development host
ifndef SMARTOS_HOST
	$(error You must define $$SMARTOS_HOST before proceeding)
else
	rsync --delete --exclude="target/*" -r . "${SMARTOS_HOST}:~/portunus"
endif

test: #: Run unit tests 
	$(call smartos, cargo test)

run: #: Stand up an instance of portunus with the included example applications
	$(call smartos, cargo run)

provision: #: Install all our dev packages
	$(call smartos, pfexec pkgin -y install clang gmake rust)

clean: #: Clean up so we can rebuild from scratch
	$(call smartos, cargo clean)

hello_web: #: Launch the hello_web example application
	$(call smartos, cargo run --example hello_web)

.PHONY: help test run provision clean hello_web sync
