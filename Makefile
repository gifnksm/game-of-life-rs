buildtype = release

PROJECT = game-of-life-rs
TARGET = asmjs-unknown-emscripten

SRV_DIR = srv
SRV_PORT = 8080

JS_FILE = $(PROJECT).js
MEM_FILE = $(subst -,_,$(PROJECT)).js.mem

CARGO_OUTDIR = target/$(TARGET)/$(buildtype)/

CARGO_OPTION = --target $(TARGET)
EMCC_OPTION = -s USE_SDL=2 -s USE_WEBGL2=1 -s FULL_ES3=1

ifeq ($(buildtype),release)
CARGO_OPTION += --release
EMCC_OPTION += -O3
SRV_FILES = $(SRV_DIR)/$(JS_FILE) $(SRV_DIR)/$(MEM_FILE)

else ifeq ($(buildtype),debug)
CARGO_OPTION +=
EMCC_OPTION += -g4
SRV_FILES = $(SRV_DIR)/$(JS_FILE)

else
$(error "unknown buildtype")
endif

all: $(SRV_FILES)
.PHONY: all

clean:
	cargo clean
	$(RM) $(SRV_DIR)/$(JS_FILE) $(SRV_DIR)/$(MEM_FILE)
.PHONY: clean

serve: $(SRV_FILES)
	ruby -run -e httpd $(SRV_DIR) -p $(SRV_PORT)

FORCE:
.PHONY: FORCE

$(CARGO_OUTDIR)/$(JS_FILE): FORCE
	EMMAKEN_CFLAGS="$(EMCC_OPTION)" cargo build $(CARGO_OPTION)

$(CARGO_OUTDIR)/$(MEM_FILE): $(CARGO_OUTDIR)/$(JS_FILE)

$(SRV_DIR)/%: $(CARGO_OUTDIR)/%
	cp $< $@
