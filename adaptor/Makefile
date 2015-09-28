CC = gcc

ifndef RTE_SDK
$(error RTE_SDK is undefined)
endif

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

LDFLAGS += -L$(DPDK_LIB_DIR)
LIBS += -lrte_eal -lrte_mbuf -lrte_timer
LIBS += -lm -lpthread -ldl -lpcap

# change fpic to fPIC if something fails
CFLAGS = -std=gnu99 -g3 -ggdb3 -O3 -Wall -Werror -m64 -march=native -Wno-unused-function -Wno-unused-but-set-variable \
	 -I$(DPDK_INC_DIR) \
	 -D_GNU_PRODURCE \
	 -fpic

SRCS = $(wildcard *.c)
OBJS = $(SRCS:.c=.o)
HEADERS = $(wildcard)
PROD = libzcsi.so

DEPS = .make.dep

# if multiple targets are specified, do them one by one */
ifneq ($(words $(MAKECMDGOALS)),1)

.NOTPARALLEL:
$(sort all $(MAKECMDGOALS)):
	@$(MAKE) --no-print-directory -f $(firstword $(MAKEFILE_LIST)) $@

else

# parallel build by default
CORES ?= $(shell nproc || echo 1)
MAKEFLAGS += -j $(CORES)

.PHONY: all clean tags cscope

all: $(DEPS) $(PROD)
	
$(DEPS): $(SRCS) $(HEADERS)
	@$(CC) $(CFLAGS) -MM $(SRCS) | sed 's|\(.*\)\.o: \(.*\)\.c|\2.o: \2.c|' > $(DEPS);

$(PROD): $(OBJS)
	echo $(LDFLAGS)
	$(CC) -shared  $(OBJS) -o $@ $(LDFLAGS) $(LIBS)

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
