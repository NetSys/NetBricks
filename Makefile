PORT ?= "0000:00:09.0"
CORE ?= 0
BASE_DIR = $(shell git rev-parse --show-toplevel)
POOL_SIZE ?= 512
GH=git@github.com:williamofockham

.PHONY: build build-all build-ex build-nb build-rel build-rel-ex clean fmt \
        init lint native run run-rel test watch watch-test

build:
	@./build.sh build

build-all:
	@./build.sh build_all

build-ex:
ifdef EXAMPLE
	@./build.sh build_example $(EXAMPLE)
else
	@./build.sh build_example
endif

build-nb:
	@./build.sh build_fmwk

build-rel:
	@./build.sh build_rel

build-rel-ex:
ifdef EXAMPLE
	@./build.sh build_example_rel $(EXAMPLE)
else
	@./build.sh build_example
endif

clean:
	@./build.sh clean

fmt:
	@./build.sh fmt

init:
	@mkdir -p $(BASE_DIR)/.git/hooks && ln -s -f $(BASE_DIR)/.hooks/pre-commit $(BASE_DIR)/.git/hooks/pre-commit
	-git clone $(GH)/utils.git
	-ln -s utils/Vagrantfile
	-git clone --recurse-submodules $(GH)/moongen.git

lint:
	@./build.sh lint

native:
	@./build.sh build_native

run:
ifdef EXAMPLE
	@./build.sh run $(EXAMPLE) -p $(PORT) -c $(CORE) --pool-size=$(POOL_SIZE)
else
	@./build.sh run
endif

run-rel:
ifdef EXAMPLE
	@./build.sh run_rel $(EXAMPLE) -p $(PORT) -c $(CORE) --pool-size=$(POOL_SIZE)
else
	@./build.sh run_rel
endif

test:
ifdef TEST
	@unset RUST_BACKTRACE
	@./build.sh build_example $(TEST)
	@./build.sh test $(TEST)
	@export RUST_BACKTRACE=1
else
	@unset RUST_BACKTRACE
	@./build.sh build
	@./build.sh test
	@export RUST_BACKTRACE=1
endif

watch:
	@cargo watch --poll -x build -w framework/src

watch-test:
	@cargo watch --poll -x test -w framework/src
