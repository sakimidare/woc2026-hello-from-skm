# SPDX-License-Identifier: GPL-2.0

KDIR ?= ./linux
BDIR ?= ./busybox
SUBMODULE_DEPTH ?= 1
TARGET ?=

.PHONY: all run build setup clean

all: run

run: 
	scripts/run.sh 

build: 
	scripts/build.sh -b $(BDIR) -k $(KDIR) -t "$(TARGET)"

setup:
	SUBMODULE_DEPTH=$(SUBMODULE_DEPTH) scripts/setup.sh

clean:
	$(MAKE) -C $(KDIR) M=$$PWD clean
	$(MAKE) -C $(BDIR) M=$$PWD clean
