# SPDX-License-Identifier: GPL-2.0

KDIR ?= ./linux
BDIR ?= ./busybox
SUBMODULE_DEPTH ?= 1
TARGET ?=
NCPU ?= $(shell nproc)

# 构建产物路径
BZIMAGE := $(KDIR)/arch/x86_64/boot/bzImage
ROOTFS := $(BDIR)/rootfs.img
BUSYBOX_BIN := $(BDIR)/busybox
BUSYBOX_INSTALL := $(BDIR)/_install

.PHONY: all run build setup clean rebuild kernel busybox rootfs config-rootfs

all: run

# 增量构建：分别检查并构建缺失的部分
build: kernel busybox rootfs

# 强制完全重新构建
rebuild: clean-build build

# 编译内核
kernel: $(BZIMAGE)

$(BZIMAGE):
	@echo "Building linux kernel..."
	@cd $(KDIR) && yes "" | make LLVM=1 CLIPPY=1 $(TARGET) -j$(NCPU) || [ $$? -eq 141 ]

# 编译 busybox
busybox: $(BUSYBOX_BIN)

$(BUSYBOX_BIN):
	@echo "Building busybox..."
	@cd $(BDIR) && yes "" | make -j$(NCPU) || [ $$? -eq 141 ]

# 安装并配置 rootfs
rootfs: $(ROOTFS)

$(ROOTFS): $(BUSYBOX_BIN)
	@echo "Installing busybox..."
	@cd $(BDIR) && make install
	@echo "Configuring rootfs..."
	@scripts/config-rootfs.sh -b $(BDIR)
	@echo "Packing rootfs image..."
	@cd $(BDIR)/_install && find . | cpio -o -H newc | gzip > ../rootfs.img

# 运行：确保构建产物存在
run: build
	@echo "Starting QEMU..."
	scripts/run.sh -b $(BDIR) -k $(KDIR)

setup:
	SUBMODULE_DEPTH=$(SUBMODULE_DEPTH) scripts/setup.sh

# 清理构建产物
clean-build:
	@echo "Cleaning build artifacts..."
	@rm -f $(BZIMAGE) $(ROOTFS)
	@rm -rf $(BUSYBOX_INSTALL)

clean: clean-build
	@echo "Cleaning kernel..."
	@$(MAKE) -C $(KDIR) clean 2>/dev/null || true
	@echo "Cleaning busybox..."
	@$(MAKE) -C $(BDIR) clean 2>/dev/null || true
