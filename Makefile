TARGET := $(shell cargo metadata --format-version=1 | jq -r .target_directory)/$(if $(RELEASE),release,debug)

CLIENT := $(TARGET)/client
SERVER := $(TARGET)/server
GFX := $(TARGET)/gfx

.PHONY: all
all: client server gfx

.PHONY: clean
clean:
	cargo clean

.PHONY: fclean
fclean: clean
	rm -f client server gfx

.PHONY: re
re:
	@$(MAKE) --no-print-directory fclean
	@$(MAKE) --no-print-directory all

client: $(CLIENT)
	cp $(CLIENT) client

$(CLIENT):
	cargo build $(if $(RELEASE),--release) --bin client

server: $(SERVER)
	cp $(SERVER) server

$(SERVER):
	RUSTFLAGS="-C panic=abort" cargo build -Z build-std=core,alloc,compiler_builtins $(if $(RELEASE),--release) --bin server

gfx: $(GFX)
	cp $(GFX) gfx

$(GFX):
	cargo build $(if $(RELEASE),--release) --bin gfx

-include $(TARGET)/client.d
-include $(TARGET)/server.d
-include $(TARGET)/gfx.d
