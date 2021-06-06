help: #: Build this help menu from Makefile target comments (default)
	@echo "USAGE:\n"
	@awk -F ':' 'NF >= 3 { OFS="#"; print "-",$$1,$$3 }' $(MAKEFILE_LIST) \
		| sort | column -s "#" -t

test: #: Run unit tests 
	cargo test

run: #: Stand up an instance of portunusd with the included example applications
	cargo run

provision: #: Install all our dev packages
	pkgin -y install clang gmake rust

clean: _smartos_host #: Clean up so we can rebuild from scratch
	cargo clean

.PHONY: help test run provision clean
