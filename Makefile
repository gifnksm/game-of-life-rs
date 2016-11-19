buildtype = release

PROJECT = game-of-life-rs
TARGET = asmjs-unknown-emscripten

DOCS_DIR = docs
DOCS_PORT = 8080

JS_FILE = $(PROJECT).js
MEM_FILE = $(subst -,_,$(PROJECT))-*.js.mem

CARGO_OUTDIR = target/$(TARGET)/$(buildtype)

CARGO_OPTION = --target $(TARGET)
EMCC_OPTION = -s USE_SDL=2

ifeq ($(buildtype),release)
CARGO_OPTION += --release
EMCC_OPTION += -O3
DOCS_FILES = $(DOCS_DIR)/$(JS_FILE) $(DOCS_DIR)/$(MEM_FILE)

else ifeq ($(buildtype),debug)
CARGO_OPTION +=
EMCC_OPTION += -g4
DOCS_FILES = $(DOCS_DIR)/$(JS_FILE)

else
$(error "unknown buildtype")
endif

all: $(DOCS_FILES)
.PHONY: all

clean:
	cargo clean
	$(RM) $(DOCS_DIR)/$(JS_FILE) $(DOCS_DIR)/$(MEM_FILE)
.PHONY: clean

serve: $(DOCS_FILES)
	ruby -run -e httpd $(DOCS_DIR) -p $(DOCS_PORT)

FORCE:
.PHONY: FORCE

$(CARGO_OUTDIR)/$(JS_FILE): FORCE
	$(RM) $(DOCS_FILES)
	EMMAKEN_CFLAGS="$(EMCC_OPTION)" cargo build $(CARGO_OPTION)

$(CARGO_OUTDIR)/$(MEM_FILE): $(CARGO_OUTDIR)/$(JS_FILE)

$(DOCS_DIR)/$(JS_FILE): $(CARGO_OUTDIR)/$(JS_FILE) FORCE
	cp $< $@

$(DOCS_DIR)/$(MEM_FILE): $(CARGO_OUTDIR)/$(JS_FILE) FORCE
	cp $(CARGO_OUTDIR)/deps/$(MEM_FILE) $(DOCS_DIR)

