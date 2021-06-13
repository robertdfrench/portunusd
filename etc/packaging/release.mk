portunusd-0.1.0.tar.gz: \
	files/sbin/portunusd files/man/man8/portunusd.8 \
	buildinfo.txt shortdesc.txt longdesc.txt packlist.txt
	pkg_create \
		-B buildinfo.txt -c shortdesc.txt -d longdesc.txt \
		-f packlist.txt -I /opt/local -p build/ \
		-U $@

files/sbin/portunusd: ../target/release/portunusd
	mkdir -p $(dir $@)
	cp $< $@

files/man/man8/portunusd.8: ../src/manual/portunusd.8
	mkdir -p $(dir $@)
	cp $< $@

%.txt: ../etc/packaging/%.txt
	cp $< $@
