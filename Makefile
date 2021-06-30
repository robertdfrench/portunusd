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
		bash -c 'true; set -e; cd portunusd; set -x; $(1)'
endif


# This target syncs code from your local workstation to a remote smartos host.
# It doesn't use the "smartos" macro defined above, because it really only makes
# sense to run from your local workstation. It requires an environment variable
# called $SMARTOS_HOST, which should contain the hostname of the remote smartos
# system.
#
sync: #: Push latest code to development host
ifndef SMARTOS_HOST
	$(error You must define $$SMARTOS_HOST before proceeding)
else
	rsync -t \
		--delete \
		--exclude="target/*" \
		-r . "${SMARTOS_HOST}:~/portunusd"
endif


# This task tracks what packages our smartos system needs in order to build
# portunus. It is idempotent, so we could provisionally run it as a
# pre-requisite of every other task, but it takes a few seconds and does not
# usually make any changes. So instead, we separate it, and run it as needed (as
# we change the package list).
provision: #: Install all our dev packages
	$(call smartos, pfexec pkgin -y install clang gmake rust)


# This task generates documentation and opens a web browser, so it should only
# run on the developer's workstation.
#
docs: #: Build documentation from Rust source files
	cargo doc --open

test: #: Run unit tests 
	$(call smartos, cargo test)

clean: #: Clean up so we can rebuild from scratch
	$(call smartos, cargo clean)

hello_web: #: Launch the hello_web example application
	$(call smartos, cargo run --example hello_web)

run: #: Run the portunusd server
	$(call smartos, cargo run)

install: #: Install PortunusD on a SmartOS host
	$(call smartos, make _install)
			
_install: \
	/opt/local/man/man8/portunusd.8.gz \
	/opt/local/man/man5/portunusd.conf.5.gz \
	/opt/local/etc/portunusd.conf \
	/opt/local/var/www/index.html \
	/opt/local/sbin/portunusd;

/opt/local/sbin/portunusd: target/release/portunusd
	install $< $@

/opt/local/var/www/index.html: etc/www/index.html
	mkdir -p $(dir $@)
	install -m 644 $< $@

.PHONY: target/release/portunusd
target/release/portunusd:
	cargo build --release

/opt/local/man/man8/portunusd.8.gz: target/release/man/portunusd.8.gz
	install -m 644 $< $@

target/release/man/portunusd.8.gz: etc/manual/portunusd.8
	mkdir -p $(dir $@)
	cat $< | gzip > $@

/opt/local/man/man5/portunusd.conf.5.gz: target/release/man/portunusd.conf.5.gz
	install -m 644 $< $@

target/release/man/portunusd.conf.5.gz: etc/manual/portunusd.conf.5
	mkdir -p $(dir $@)
	cat $< | gzip > $@

/opt/local/etc/portunusd.conf: etc/portunusd.conf
	install -m 644 $< $@

.PHONY: help provision docs test run clean hello_web sync install
