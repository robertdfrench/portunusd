SOURCE_FILES = $(wildcard *.c)

all: $(patsubst %.c,%.app,$(SOURCE_FILES));

%.app: %.c
	clang $< -o $@

start_all: $(patsubst %.c,%.start,$(SOURCE_FILES));

%.start: %.app %.stop
	(./$< &)

stop_all: $(patsubst %.c,%.stop,$(SOURCE_FILES));

%.stop:
	ps aux \
		| awk '/$*/ && !/awk/ { print $$2 }' \
		| xargs -n1 --no-run-if-empty kill -9
	rm -f /var/run/$*.pid /var/run/$*_door

clean: stop_all
	rm -f *.app
