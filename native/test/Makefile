CC = gcc

RTE_SDK = $(abspath ../../3rdparty/dpdk/)

ifndef RTE_TARGET
$(error RTE_TARGET is undefined)
endif

ifneq ($(wildcard $(RTE_SDK)/$(RTE_TARGET)*),)
	DPDK_INC_DIR = $(RTE_SDK)/$(RTE_TARGET)/include
	DPDK_LIB_DIR = $(RTE_SDK)/$(RTE_TARGET)/lib
else
	DPDK_INC_DIR = $(RTE_SDK)/build/include
	DPDK_LIB_DIR = $(RTE_SDK)/build/lib
endif
ADAPTOR_DIR=$(abspath ../)
DPDK_INCLUDE=$(ADAPTOR_DIR)/include
DPDK_LIB=$(ADAPTOR_DIR)

LDFLAGS += -L$(DPDK_LIB_DIR) -L$(DPDK_LIB) -Wl,-rpath=$(DPDK_LIB)\
	   -Wl,-rpath=$(DPDK_LIB_DIR)
LIBS += -lm -lpthread -ldl -lpcap -ldpdk -lzcsi


# change fpic to fPIC if something fails
CFLAGS = -std=gnu99 -g3 -ggdb3 -O3 -Wall -Werror -m64 -march=native \
	 -Wno-unused-function -Wno-unused-but-set-variable \
	 -I$(DPDK_INC_DIR) \
	 -I$(DPDK_INCLUDE) \
	 -D_GNU_SOURCE

SRCS = $(wildcard *.c)
OBJS = $(SRCS:.c=.o)
HEADERS = $(wildcard include/*.h)
PROD = test

DEPS = .make.dep

OSTYPE ?= $(shell uname)
ifeq ($(OSTYPE),Darwin)
	MAKEJOBS := $(shell sysctl -n hw.physicalcpu)
else
	MAKEJOBS := 1
endif

# if multiple targets are specified, do them one by one */
ifneq ($(words $(MAKECMDGOALS)),1)

.NOTPARALLEL:
$(sort all $(MAKECMDGOALS)):
	@$(MAKE) --no-print-directory -f $(firstword $(MAKEFILE_LIST)) $@

else

# parallel build by default
CORES ?= $(shell nproc || echo $(MAKEJOBS))
MAKEFLAGS += -j $(CORES)

.PHONY: all clean tags cscope

all: $(DEPS) $(PROD)

$(DEPS): $(SRCS) $(HEADERS)
	@$(CC) $(CFLAGS) -MM $(SRCS) | sed 's|\(.*\)\.o: \(.*\)\.c|\2.o: \2.c|' > $(DEPS);

$(PROD): $(OBJS)
	echo $(LDFLAGS)
	$(CC) $(OBJS) -o $@ $(LDFLAGS) $(LIBS)

-include $(DEPS)

clean:
	rm -f $(DEPS) $(PROD) *.o

tags:
	@ctags -R *

cscope:
	@rm -f cscope.*
	@find . -name "*.c" -o -name "*.h" > cscope.files
	cscope -b -q -k
	@rm -f cscope.files
endif
