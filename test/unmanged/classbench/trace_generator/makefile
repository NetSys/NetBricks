## makefile for trace_generator
## David E. Taylor 
## Applied Research Laboratory
## Department of Computer Science and Engineering
## Washington University in Saint Louis
## det3@arl.wustl.edu
##
CC = g++
CFLAGS = -g -pg
.cc.o:
	${CC} ${CFLAGS} -c $*.cc 

hlist.o : stdinc.h hlist.h
dlist.o : stdinc.h hlist.h
flist.o : stdinc.h hlist.h
FilterList.o : stdinc.h FilterList.h
db_parser.o : stdinc.h FilterList.h db_parser.h
trace_tools.o : stdinc.h dlist.h flist.h hlist.h FilterList.h trace_tools.h
trace_generator.o : stdinc.h db_parser.h trace_tools.h

trace_generator:	trace_generator.o db_parser.o trace_tools.o hlist.o dlist.o flist.o FilterList.o
	${CC} ${CFLAGS} trace_generator.o db_parser.o trace_tools.o hlist.o dlist.o flist.o FilterList.o -o trace_generator

all:	trace_generator

