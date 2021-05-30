# I do most of my development work on an ancient, non-illumos laptop. It isn't
# quite beefy enough to run a VM, so I target remote SmartOS zones hosted by
# https://mnx.io . This Makefile is intended to be invoked from my laptop, and
# allows me to build and test against $SMARTOS_HOST, which is presumed to be
# some illumos distro with pkgsrc installed.
#
# If you are already on an illumos host, just hop into src/ and run `make` from
# there -- that Makefile is intended to run on illumos.

help: #: Build this help menu from Makefile target comments (default)
	@echo "USAGE:\n"
	@awk -F ':' 'NF >= 3 { OFS="#"; print "-",$$1,$$3 }' $(MAKEFILE_LIST) \
		| sort | column -s "#" -t

check: sync #: Test latest code on $SMARTOS_HOST
	ssh root@$(SMARTOS_HOST) gmake -C relaydoors/src check

provision: sync #: Install all our dev packages on $SMARTOS_HOST
	ssh root@$(SMARTOS_HOST) pkgin -y install clang gmake

sync: _smartos_host #: Sync our code to $SMARTOS_HOST
	rsync -r . root@$(SMARTOS_HOST):~/relaydoors

clean: _smartos_host #: Remove our code from $SMARTOS_HOST
	ssh root@$(SMARTOS_HOST) rm -rf relaydoors

_smartos_host: # throw an error if $SMARTOS_HOST isn't defined
ifndef SMARTOS_HOST
	$(error "You must set $$SMARTOS_HOST to the remote zone's IP address")
endif

.PHONY: help test provision sync clean _smartos_host
