.PHONY: start
start: stop start_examples
	cd .. && cargo run

.PHONY: test
test:
	cd .. && cargo test

.PHONY: start_examples
start_examples:
	$(MAKE) -C examples start_all

.PHONY: clean
clean: stop
	$(MAKE) -C examples clean
	rm -f relaydoors

.PHONY: stop
stop:
	ps aux \
		| awk '/relaydoors/ && !/awk/ { print $$2 }' \
		| xargs -n1 --no-run-if-empty kill -9
