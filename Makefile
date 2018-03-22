PORT ?= "0000:00:08.0"
CORE ?= 1

.PHONY: clean compile compile-all fmt lint release release-all run run-rel test

compile:
ifdef TEST
	@./build.sh build_test $(TEST)
else
	@./build.sh build_test
endif

compile-all:
	@./build.sh build

release-all:
	@./build.sh build_rel

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
	@./build.sh run $(TEST) -p $(PORT) -c $(CORE)
else
	@./build.sh run
endif

run-rel:
ifdef TEST
	@./build.sh run_rel $(TEST) -p $(PORT) -c $(CORE)
else
	@./build.sh run_rel
endif

test:
	@./build.sh test
