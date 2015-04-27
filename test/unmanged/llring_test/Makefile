
LIBS=-lpthread
CFLAGS=-O3
CC=gcc

.PHONY=all test

all: test

clean:
	rm test

test: test.c llring.h
	$(CC) -c $< -o $@ $(CFLAGS) $(LIBS)
