test: run/server.pid run/server.door build/client /home/alice/README.md /home/bob/README.md
	./build/client --first-door run/server.door --username alice
	./build/client --first-door run/server.door --username alice
	./build/client --first-door run/server.door --username bob
	./build/client --first-door run/server.door --username bob
	sudo kill -9 `cat run/server.pid` || true
	sudo rm -f run/server.*

/home/alice/README.md: alice.md /home/alice/.profile
	sudo install -o alice -m 644 alice.md /home/alice/README.md

/home/bob/README.md: bob.md /home/bob/.profile
	sudo install -o bob -m 644 bob.md /home/bob/README.md

/home/alice/.profile:
	sudo useradd -m alice

/home/bob/.profile:
	sudo useradd -m bob

run/server.pid: run/server.door
	@cat run/server.pid

run/server.door: build/server run/.dir
	sudo ./build/server --pid run/server.pid --door run/server.door

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
	sudo rm -rf build/ run/
