test: run/server.pid run/server.door build/client
	./build/client --first-door run/server.door || true
	kill -9 `cat run/server.pid` || true
	rm -f run/server.*

run/server.pid: run/server.door
	@cat run/server.pid

run/server.door: build/server run/.dir
	./build/server --pid run/server.pid --door run/server.door

build/client: client.c build/.dir
	gcc -o build/client -lsocket -lnsl client.c

build/server: server.c build/.dir
	gcc -o build/server -lsocket -lnsl server.c

build/.dir:
	mkdir -p build
	touch build/.dir

run/.dir:
	mkdir -p run
	touch run/.dir

clean:
	rm -rf build/ run/
