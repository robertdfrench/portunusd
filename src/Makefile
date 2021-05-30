check: hello_web relaydoors
	./relaydoors

relaydoors: relaydoors.c
	clang $< -o $@

hello_web:
	$(MAKE) -C examples hello_web.test
