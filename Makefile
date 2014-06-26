# Rust-Empty: An Makefile to get started with Rust
# https://github.com/bvssvni/rust-empty
# 
# The MIT License (MIT)
#
# Copyright (c) 2014 Sven Nilsen
#
# Permission is hereby granted, free of charge, to any person obtaining a copy of
# this software and associated documentation files (the "Software"), to deal in
# the Software without restriction, including without limitation the rights to
# use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
# the Software, and to permit persons to whom the Software is furnished to do so,
# subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
# FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
# COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
# IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
# CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

SHELL := /bin/bash

# The default make command.
# Change this to 'lib' if you are building a library.
DEFAULT = lib test doc examples
# The entry file of library source.
# Change this to support multi-crate source structure.
# For advanced usage, you can rename the file 'rust-empty.mk'
# and call it with 'make -f rust-empty.mk <command>' from your Makefile.
LIB_ENTRY_FILE = src/lib.rs
# The entry file of executable source.
EXE_ENTRY_FILE = src/main.rs

EXAMPLE_FILES = examples/*.rs
SOURCE_FILES = $(shell test -e src/ && find src -type f)

COMPILER = rustc

# For release:
  COMPILER_FLAGS = --opt-level=3
# For debugging:
# COMPILER_FLAGS = -g

RUSTDOC = rustdoc

# Extracts target from rustc.
TARGET = $(shell rustc --version 2> /dev/null | awk "/host:/ { print \$$2 }")
# TARGET = x86_64-unknown-linux-gnu
# TARGET = x86_64-apple-darwin

TARGET_LIB_DIR = target
TARGET_DEP_DIR = target/deps

# Ask 'rustc' the file name of the library and use a dummy name if the source has not been created yet.
# The dummy file name is used to trigger the creation of the source first time.
# Next time 'rustc' will return the right file name.
RLIB_FILE = $(shell (rustc --crate-type=rlib --crate-file-name "$(LIB_ENTRY_FILE)" 2> /dev/null) || (echo "dummy.rlib"))
# You can't have quotes around paths because 'make' doesn't see it exists.
RLIB = $(TARGET_LIB_DIR)/$(RLIB_FILE)
DYLIB_FILE = $(shell (rustc --crate-type=dylib --crate-file-name "$(LIB_ENTRY_FILE)" 2> /dev/null) || (echo "dummy.dylib"))
DYLIB = $(TARGET_LIB_DIR)/$(DYLIB_FILE)

# Use 'VERBOSE=1' to echo all commands, for example 'make help VERBOSE=1'.
ifdef VERBOSE
  Q :=
else
  Q := @
endif

all: $(DEFAULT)

help:
	$(Q)echo "--- rust-empty (0.5 000)" \
	&& echo "make lib               - Run cargo build" \
	&& echo "make test              - Tests library internally and externally" \
	&& echo "make test-internal     - Tests library internally" \
	&& echo "make test-external     - Tests library externally" \
	&& echo "make bench             - Benchmarks library internally and externally" \
	&& echo "make bench-internal    - Benchmarks library internally" \
	&& echo "make bench-external    - Benchmarks library externally" \
	&& echo "make doc               - Builds documentation for library" \
	&& echo "make examples          - Builds examples" \
	&& echo "make rusti             - Setup 'rusti.sh' for interactive Rust" \
	&& echo "make watch             - Setup 'watch.sh' for compilation on save" \
	&& echo "make loc               - Count lines of code in src folder" \
	&& echo "make nightly-install   - Installs Rust nightly built" \
	&& echo "make nightly-uninstall - Uninstalls Rust nightly built" \
	&& echo "make clean             - Deletes binaries and documentation."

.PHONY: \
		bench \
		bench-internal \
		bench-external \
		clean \
		loc \
		nightly-install \
		nightly-uninstall \
		rusti \
		test \
		test-internal \
	  test-external \
		watch

nightly-install:
	$(Q)cd ~ \
	&& curl -s http://www.rust-lang.org/rustup.sh > rustup.sh \
	&& ( \
		echo "Rust install-script stored as '~/rustup.sh'" ; \
		read -p "Do you want to install? [y/n]:" -n 1 -r ; \
		echo "" ; \
		if [[ $$REPLY =~ ^[Yy]$$ ]] ; \
		then \
			cat rustup.sh | sudo sh ; \
		fi \
	)

nightly-uninstall:
	$(Q)cd ~ \
	&& curl -s http://www.rust-lang.org/rustup.sh > rustup.sh \
	&& ( \
		echo "Rust install-script stored as '~/rustup.sh'" ; \
		read -p "Do you want to uninstall? [y/n]:" -n 1 -r ; \
		echo "" ; \
		if [[ $$REPLY =~ ^[Yy]$$ ]] ; \
		then \
			cat rustup.sh | sudo sh -s -- --uninstall ; \
		fi \
	)

doc: $(SOURCE_FILES) | src/
	$(Q)$(RUSTDOC) $(LIB_ENTRY_FILE) -L "$(TARGET_LIB_DIR)" -L "$(TARGET_DEP_DIR)" \
	&& echo "--- Built documentation"

target-dir: $(TARGET_LIB_DIR)

bin/main: $(SOURCE_FILES) | bin/ $(EXE_ENTRY_FILE)
	$(Q)$(COMPILER) $(COMPILER_FLAGS) $(EXE_ENTRY_FILE) -o bin/main -L "$(TARGET_LIB_DIR)" -L "$(TARGET_DEP_DIR)" \
	&& echo "--- Built executable" \
	&& echo "--- Type 'make run' to run executable"

test: test-internal test-external
	$(Q)echo "--- Internal tests succeeded" \
	&& echo "--- External tests succeeded"

test-external: bin/test-external
	$(Q)cd "bin/" \
	&& ./test-external

bin/test-external: $(SOURCE_FILES) | rlib bin/ src/test.rs
	$(Q)$(COMPILER) $(COMPILER_FLAGS) --test src/test.rs -o bin/test-external -L "$(TARGET_LIB_DIR)" -L "$(TARGET_DEP_DIR)" \
	&& echo "--- Built external test runner"

test-internal: bin/test-internal
	$(Q)cd "bin/" \
	&& ./test-internal

bin/test-internal: $(SOURCE_FILES) | rlib src/ bin/
	$(Q)$(COMPILER) $(COMPILER_FLAGS) --test $(LIB_ENTRY_FILE) -o bin/test-internal -L "$(TARGET_LIB_DIR)" -L "$(TARGET_DEP_DIR)" \
	&& echo "--- Built internal test runner"

bench: bench-internal bench-external

bench-external: test-external
	$(Q)bin/test-external --bench

bench-internal: test-internal
	$(Q)bin/test-internal --bench

lib: rlib
	$(Q)echo "--- Type 'make test' to test library"

rlib: $(RLIB)

$(RLIB): $(SOURCE_FILES) | $(LIB_ENTRY_FILE) $(TARGET_LIB_DIR)
	cargo build \
	&& echo "--- Built rlib"

bin/:
	$(Q)mkdir -p bin

$(TARGET_LIB_DIR):
	$(Q)mkdir -p $(TARGET_LIB_DIR)

src/:
	$(Q)mkdir -p src

examples-dir:
	$(Q)test -e examples \
	|| \
	( \
		mkdir examples \
		&& echo -e "fn main() {\n\tprintln!(\"Hello!\");\n}\n" > examples/hello.rs \
		&& echo "--- Created examples folder" \
	)

rust-dir:
	$(Q)mkdir -p .rust

examples: $(EXAMPLE_FILES)

$(EXAMPLE_FILES): lib examples-dir
	$(Q)$(COMPILER) $(COMPILER_FLAGS) $@ -L "$(TARGET_LIB_DIR)" -L "$(TARGET_DEP_DIR)" --out-dir examples/ \
	&& echo "--- Built examples"

$(EXE_ENTRY_FILE): | src/
	$(Q)test -e $(EXE_ENTRY_FILE) \
	|| \
	( \
		echo -e "fn main() {\n\tprintln!(\"Hello world!\");\n}" > $(EXE_ENTRY_FILE) \
	)

src/test.rs: | src/
	$(Q)test -e src/test.rs \
	|| \
	( \
		touch src/test.rs \
	)

$(LIB_ENTRY_FILE): | src/
	$(Q)test -e $(LIB_ENTRY_FILE) \
	|| \
	( \
		echo -e "#![crate_id = \"\"]\n#![deny(missing_doc)]\n\n//! Documentation goes here.\n" > $(LIB_ENTRY_FILE) \
	)

clean:
	$(Q)rm -f "$(RLIB)"
	$(Q)rm -f "$(DYLIB)"
	$(Q)rm -rf "doc/"
	$(Q)rm -f "bin/main"
	$(Q)rm -f "bin/test-internal"
	$(Q)rm -f "bin/test-external"
	$(Q)echo "--- Deleted binaries and documentation"

# borrowed from http://stackoverflow.com/q/649246/1256624
define RUSTI_SCRIPT
#!/bin/bash

#written by mcpherrin

while true; do
  echo -n "> "
  read line
  TMP="`mktemp r.XXXXXX`"
  $(COMPILER) - -o $$TMP -L "$(TARGET_LIB_DIR)/" -L "$(TARGET_DEP_DIR)" <<EOF
  #![feature(globs, macro_rules, phase, struct_variant)]
  extern crate arena;
  extern crate collections;
  extern crate flate;
  #[phase(syntax)] extern crate fourcc;
  extern crate glob;
  extern crate green;
  extern crate hexfloat;
  extern crate libc;
  #[phase(syntax, link)] extern crate log;
  extern crate native;
  extern crate num;
  extern crate rand;
  extern crate regex;
  #[phase(syntax)] extern crate regex_macros;
  extern crate rustc;
  extern crate rustdoc;
  extern crate rustuv;
  extern crate semver;
  extern crate serialize;
  extern crate sync;
  extern crate syntax;
  extern crate term;
  extern crate test;
  extern crate time;
  extern crate url;
  extern crate uuid;
  extern crate workcache;

  fn main() {
      let r = { $$line };
      println!("{:?}", r);
  }
EOF
  ./$$TMP
  rm $$TMP
done
endef
export RUSTI_SCRIPT

rusti: $(TARGET_LIB_DIR)
	$(Q)( \
		test -e rusti.sh \
		&& echo "--- The file 'rusti.sh' already exists" \
	) \
	|| \
	( \
		echo -e "$$RUSTI_SCRIPT" > rusti.sh \
		&& chmod +x rusti.sh \
		&& echo "--- Created 'rusti.sh'" \
		&& echo "--- Type './rusti.sh' to start interactive Rust" \
	)

# borrowed from http://stackoverflow.com/q/649246/1256624
define WATCH_SCRIPT
#!/bin/bash

#written by zzmp

# This script will recompile a rust project using `make`
# every time something in the specified directory changes.

# Watch files in infinite loop
watch () {
  if [ -e "$$2" ]; then
    echo "Watching files in $$2.."
    CTIME=$$(date -j -f "%a %b %d %T %Z %Y" "`date`" "+%s")
    while :; do
      sleep 1
      for f in `find $$2 -type f -name "*.rs"`; do
        eval $$(stat -s $$f)
        if [ $$st_mtime -gt $$CTIME ]; then
          CTIME=$$(date -j -f "%a %b %d %T %Z %Y" "`date`" "+%s")
          echo "~~~ Rebuilding"
          $$1
          if [ ! $$? -eq 0 ]; then
            echo ""
          fi
        fi
      done
    done
  else
    echo "$$2 is not a valid directory"
  fi
}

# Capture user input with defaults
CMD=$${1:-make}
DIR=$${2:-src}

if [ $${CMD:0:2} = '-h' ]; then
echo '
This script will recompile a rust project using `make`
every time something in the specified directory changes.

Use: ./watch.sh [CMD] [DIR]
Example: ./watch.sh "make run" src

CMD: Command to execute
     Complex commands may be passed as strings
     `make` by default
DIR: Directory to watch
     src by default

If DIR is supplied, CMD must be as well.\n'
else
  watch "$$CMD" "$$DIR"
fi

endef
export WATCH_SCRIPT

watch: $(TARGET_LIB_DIR)
	$(Q)( \
		test -e watch.sh \
		&& echo "--- The file 'watch.sh' already exists" \
	) \
	|| \
	( \
		echo -e "$$WATCH_SCRIPT" > watch.sh \
		&& chmod +x watch.sh \
		&& echo "--- Created 'watch.sh'" \
		&& echo "--- Type './watch.sh' to start compilation on save" \
		&& echo "--- Type './watch.sh -h' for more options" \
	)

loc:
	$(Q)echo "--- Counting lines of .rs files in 'src' (LOC):" \
	&& find src/ -type f -name "*.rs" -exec cat {} \; | wc -l
