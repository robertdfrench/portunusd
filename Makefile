help: #: Build this help menu from Makefile target comments (default)
	@echo "USAGE:\n"
	@awk -F ':' 'NF >= 3 { OFS="#"; print "-",$$1,$$3 }' $(MAKEFILE_LIST) \
		| sort | column -s "#" -t

test: #: Run unit tests 
	cargo test

run: #: Stand up an instance of portunus with the included example applications
	cargo run

provision: #: Install all our dev packages
	pkgin -y install clang gmake rust

clean: #: Clean up so we can rebuild from scratch
	cargo clean

hello_web: #: Launch the hello_web example application
	cargo run --example hello_web

.PHONY: help test run provision clean hello_web
