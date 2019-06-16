TARGET_ARCH := xtensa-esp32-none-elf

$(COMPONENT_LIBRARY): $(COMPONENT_PATH)/target/$(TARGET_ARCH)/release/librust_main.a

$(COMPONENT_PATH)/target/$(TARGET_ARCH)/release/librust_main.a: $(COMPONENT_PATH)/Cargo.toml $(wildcard $(COMPONENT_PATH)/src/*.rs) $(wildcard $(COMPONENT_PATH)/idf/src/*.rs) $(wildcard $(COMPONENT_PATH)/idf/wrapper.h) $(wildcard $(COMPONENT_PATH)/idf/build.rs) $(wildcard $(COMPONENT_PATH)/m5stack/src/*.rs) $(wildcard $(COMPONENT_PATH)/peripheral/src/*.rs)
	cd $(COMPONENT_PATH); xargo build --target $(TARGET_ARCH) --release

COMPONENT_ADD_LDFLAGS := -L$(COMPONENT_PATH)/target/$(TARGET_ARCH)/release -lrust_main

COMPONENT_EXTRA_CLEAN := $(COMPONENT_PATH)/target
