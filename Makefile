PORT ?= "0000:00:09.0"
CORE ?= 0
BASE_DIR = $(shell git rev-parse --show-toplevel)
POOL_SIZE ?= 512

.PHONY: init compile compile-nb compile-test native build-rel build-rel-test \
        fmt clean lint run run-rel test

init:
	@mkdir -p $(BASE_DIR)/.git/hooks && ln -s -f $(BASE_DIR)/.hooks/pre-commit $(BASE_DIR)/.git/hooks/pre-commit

compile:
	@./build.sh build

compile-nb:
	@./build.sh build_fmwk

compile-test:
ifdef TEST
	@./build.sh build_test $(TEST)
else
	@./build.sh build_test
endif

native:
	@./build.sh build_native

build-rel:
	@./build.sh build_rel

build-rel-test:
ifdef TEST
	@./build.sh build_test_rel $(TEST)
else
	@./build.sh build_test
endif

fmt:
	@./build.sh fmt

clean:
	@./build.sh clean

# Clippy has some issues current with Rust nightly. Will keep an eye on this
# development.
lint:
	@./build.sh lint

run:
ifdef TEST
	@./build.sh run $(TEST) -p $(PORT) -c $(CORE) --pool_size=$(POOL_SIZE)
else
	@./build.sh run
endif

run-rel:
ifdef TEST
	@./build.sh run_rel $(TEST) -p $(PORT) -c $(CORE) --pool_size=$(POOL_SIZE)
else
	@./build.sh run_rel
endif

test:
ifdef TEST
	@./build.sh build_test $(TEST)
	@./build.sh test $(TEST)
else
	@./build.sh build
	@./build.sh test
endif
