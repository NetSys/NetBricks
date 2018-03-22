CC = gcc

ifndef RTE_SDK
	$(error RTE_SDK is undefined)
endif

ifndef RTE_TARGET
	RTE_TARGET=build
endif

ifneq ($(wildcard $(RTE_SDK)/$(RTE_TARGET)*),)
	DPDK_INC_DIR = $(RTE_SDK)/$(RTE_TARGET)/include
	DPDK_LIB_DIR = $(RTE_SDK)/$(RTE_TARGET)/lib
else
	DPDK_INC_DIR = $(RTE_SDK)/build/include
	DPDK_LIB_DIR = $(RTE_SDK)/build/lib
endif

LDFLAGS += -L$(DPDK_LIB_DIR)
LIBS += -Wl,--whole-archive -ldpdk -Wl,--no-whole-archive -Wl,-rpath=$(DPDK_LIB_DIR)
#LIBS += -ldpdk -Wl,-rpath=$(DPDK_LIB_DIR)
LIBS += -lm -lpthread -ldl -lpcap -lnuma

# change fpic to fPIC if something fails
CFLAGS = -std=gnu99 -g3 -ggdb3 -O3 -Wall -Werror -m64 -march=native \
	 -Wno-unused-function -Wno-unused-but-set-variable \
	 -I$(DPDK_INC_DIR) -Iinclude/\
	 -D_GNU_SOURCE \
	 -fPIC

SRCS = $(wildcard *.c)
OBJS = $(SRCS:.c=.o)
HEADERS = $(wildcard include/*.h)
PROD = libzcsi.so
PROD_STATIC = libzcsi.a

DEPS = .make.dep

ifeq ($(OSTYPE),Darwin)
	MAKEJOBS := $(shell sysctl -n hw.physicalcpu)
else
	MAKEJOBS := $(shell nproc || echo 1)
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
INSTALL_PATH = $(abspath ../target/native)

.PHONY: all clean tags cscope all-static

all: $(DEPS) $(PROD)

all-static: $(DEPS) $(PROD_STATIC)

install: $(DEPS) $(PROD) | $(INSTALL_PATH)
	cp $(PROD) $(INSTALL_PATH)

$(INSTALL_PATH):
	mkdir -p $(INSTALL_PATH)

$(DEPS): $(SRCS) $(HEADERS)
	@echo $(RTE_SDK) $(DPDK_INC_DIR)
	@$(CC) $(CFLAGS) -MM $(SRCS) | sed 's|\(.*\)\.o: \(.*\)\.c|\2.o: \2.c|' > $(DEPS);

$(PROD_STATIC): $(OBJS)
	ar rcs $(PROD_STATIC) $(OBJS)

$(PROD): $(OBJS)
	$(CC) -shared  $(OBJS) -o $@ $(LDFLAGS) $(LIBS)

-include $(DEPS)

clean:
	rm -f $(DEPS) $(PROD) $(PROD_STATIC) *.o || true
	rm -f $(INSTALL_PATH)/$(PROD) || true
	rmdir $(INSTALL_PATH) || true

tags:
	@ctags -R *

cscope:
	@rm -f cscope.*
	@find . -name "*.c" -o -name "*.h" > cscope.files
	cscope -b -q -k
	@rm -f cscope.files
endif
