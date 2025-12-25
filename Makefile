# SPDX-License-Identifier: GPL-2.0

KDIR ?= ./linux
BDIR ?= ./busybox
TDIR ?= ./tools
SUBMODULE_DEPTH ?= 1
TARGET ?=
NCPU ?= $(shell nproc)

# 构建产物路径
BZIMAGE := $(KDIR)/arch/x86_64/boot/bzImage
ROOTFS := $(BDIR)/rootfs.img
BUSYBOX_BIN := $(BDIR)/busybox
BUSYBOX_INSTALL := $(BDIR)/_install

# 用户态程序名称
USERSPACE_PROG := play_tetris

.PHONY: all run build setup clean rebuild kernel busybox rootfs module module-clean module-install tools tools-clean tools-install

all: run

# 增量构建：分别检查并构建缺失的部分
build: kernel busybox module-install tools-install rootfs

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

# Rust 模块相关变量
MODULE_NAME := woc2026_hello_from_skm
MODULE_SRC := src
MODULE_KO := $(MODULE_SRC)/$(MODULE_NAME).ko

# 构建 Rust 模块
module: kernel
	@echo "Preparing Rust environment..."
	@$(MAKE) -C $(KDIR) LLVM=1 CLIPPY=1 prepare
	@echo "Building Rust kernel module..."
	$(MAKE) -C $(KDIR) M=$(PWD)/$(MODULE_SRC) LLVM=1 modules

# 清理模块
module-clean:
	@echo "Cleaning Rust module..."
	$(MAKE) -C $(KDIR) M=$(PWD)/$(MODULE_SRC) clean

# 安装模块到 rootfs
module-install: module rootfs
	@echo "Installing module to rootfs..."
	@mkdir -p $(BUSYBOX_INSTALL)/lib/modules
	@cp $(MODULE_KO) $(BUSYBOX_INSTALL)/lib/modules/
	@echo "Repacking rootfs..."
	@cd $(BUSYBOX_INSTALL) && find . | cpio -o -H newc | gzip > ../rootfs.img

# 编译用户空间工具
tools: $(TDIR)/$(USERSPACE_PROG).c
	@echo "Building userspace program..."
	@gcc -static -o $(TDIR)/$(USERSPACE_PROG).a $(TDIR)/$(USERSPACE_PROG).c -Wall

# 清理用户空间工具
tools-clean:
	@echo "Cleaning userspace program..."
	@rm -f $(TDIR)/$(USERSPACE_PROG).a

# 安装工具到 rootfs
tools-install: tools
	@echo "Installing userspace tools..."
	@cp $(TDIR)/$(USERSPACE_PROG).a $(BUSYBOX_INSTALL)/bin/$(USERSPACE_PROG)
	@cp $(TDIR)/$(USERSPACE_PROG).a $(BUSYBOX_INSTALL)/usr/bin/$(USERSPACE_PROG)
	@echo "Repacking rootfs..."
	@cd $(BUSYBOX_INSTALL) && find . | cpio -o -H newc | gzip > ../rootfs.img

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

clean: clean-build module-clean tools-clean
	@echo "Cleaning kernel..."
	@$(MAKE) -C $(KDIR) clean 2>/dev/null || true
	@echo "Cleaning busybox..."
	@$(MAKE) -C $(BDIR) clean 2>/dev/null || true
