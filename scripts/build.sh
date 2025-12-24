#!/bin/bash

set -euo pipefail

usage() {
	echo "Usage: $0 [-b /path/to/busybox] [-k /path/to/linux] [-t <target>]" 1>&2
	exit 1
}

config_rootfs() {
	pushd _install >/dev/null
	mkdir -p usr/share/udhcpc/ etc/init.d/ sbin
    # Create init symlink if it doesn't exist
    if [ ! -e sbin/init ]; then
        ln -s ../bin/busybox sbin/init
    fi
	
	cat <<EOF >etc/init.d/rcS
#!/bin/sh
mkdir -p /proc /sys /dev
mount -t proc none /proc
mount -t sysfs none /sys
mount -t devtmpfs none /dev
mkdir -p /dev/pts
mount -t devpts nodev /dev/pts
ifconfig lo up
ifconfig eth0 up
udhcpc -i eth0
telnetd -l /bin/sh
clear
EOF
	chmod a+x etc/init.d/rcS bin/* sbin/*

	cat <<EOF >etc/inittab
::sysinit:/etc/init.d/rcS
::once:-/bin/sh
::ctrlaltdel:/sbin/reboot
::ctrlaltbreak:/sbin/poweroff
#::shutdown:/bin/umount -a -r
#::shutdown:/sbin/swapoff -a
EOF
	# cp ../examples/inittab etc/

	cp ../examples/udhcp/simple.script usr/share/udhcpc/default.script
	popd >/dev/null
}

KERNEL=./linux
BUSYBOX=./busybox
TARGET=

while getopts b:k:t: option; do
	case "$option" in
	b) BUSYBOX=${OPTARG} ;;
	k) KERNEL=${OPTARG} ;;
	t) TARGET=${OPTARG} ;;
	*) usage ;;
	esac
done

if [[ ! -d "$KERNEL" ]]; then
	echo "Kernel directory '$KERNEL' not found" >&2
	exit 1
fi

echo "Building linux kernel..."
pushd "$KERNEL" >/dev/null
yes "" | make LLVM=1 CLIPPY=1 "$TARGET" -j"$(nproc)" || [ $? -eq 141 ]
popd >/dev/null

echo "Building busybox initrd..."
pushd "$BUSYBOX" >/dev/null
yes "" | make -j"$(nproc)" || [ $? -eq 141 ]
make install

echo "Setting up the rootfs"
config_rootfs

echo "Packing rootfs image"
pushd _install >/dev/null
find . | cpio -o -H newc | gzip > ../rootfs.img
popd >/dev/null
popd >/dev/null

echo "Done, you can now run 'make run'"
