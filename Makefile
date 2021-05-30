help: #: Print this help menu
	@echo "USAGE\n"
	@awk -F ':' 'NF >= 3 { OFS="#"; print $$1,$$3 }' $(MAKEFILE_LIST) \
		| column -s "#" -t

test: packages #: Test latest code on the remote host
	ssh root@$(SMARTOS_HOST) gmake -C relaydoors/src test

packages: rsync #: Install all our dev packages on the remote Zone
	ssh root@$(SMARTOS_HOST) pkgin -y install clang gmake

rsync: _smartos_host #: Sync our code to the remote Zone
	rsync -r . root@$(SMARTOS_HOST):~/relaydoors

clean: _smartos_host #: Remove our code from the remote Zone
	ssh root@$(SMARTOS_HOST) rm -rf relaydoors

_smartos_host:
ifndef SMARTOS_HOST
	$(error "You must define $$SMARTOS_HOST before proceeding")
endif

.PHONY: help test packages rsync clean _smartos_host
