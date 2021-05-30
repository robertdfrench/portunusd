all: hello_web.app
	@echo "All application examples are ready."

%.test: %.app
	kill -9 `cat /var/run/hello_web.pid` || true
	rm -f /var/run/hello_web_door
	(./$< &)

%.app: %.c
	clang $< -o $@
