.PHONY: test
test: stop test_examples relaydoors
	./relaydoors

relaydoors: relaydoors.c
	clang -lsocket -lnsl $< -o $@

.PHONY: test_examples
test_examples:
	$(MAKE) -C examples test_all

.PHONY: clean
clean: stop
	$(MAKE) -C examples clean
	rm -f relaydoors

.PHONY: stop
stop:
	ps aux \
		| awk '/relaydoors/ && !/awk/ { print $$2 }' \
		| xargs -n1 --no-run-if-empty kill -9
