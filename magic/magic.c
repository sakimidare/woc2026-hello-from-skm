#include <fcntl.h>
#include <stdio.h>
#include <sys/ioctl.h>

#define MAGIC_DEV "/dev/magic"
#define IOCTL_COMMAND_NUMBER (0x1337)

int main() {
    int fd = open(MAGIC_DEV, O_RDWR);
    if(fd == -1){
        printf("Error occurred when opening %s. Does this device exist?", MAGIC_DEV);
        return 1;
    } 

    if (ioctl(fd, IOCTL_COMMAND_NUMBER, 0) != 0) {
        printf("Error occurred when calling ioctl.\n");
        return 1;
    }

    printf("Run `dmesg | tail` to get the flag!\n");
    return 0;
    // 然后在 shell 里：dmesg | tail
}