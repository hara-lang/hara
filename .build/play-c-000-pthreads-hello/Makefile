CC = gcc
CFLAGS = -I.

out/%.o: src/%.c
	@mkdir -p out
	$(CC) -c -o $@ $< $(CFLAGS)

package: out/pthreads_hello.o
	@mkdir -p bin
	$(CC) -o bin/pthreads_hello out/pthreads_hello.o -lpthread

run: package
	./bin/pthreads_hello