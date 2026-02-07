FROM alpine:latest
RUN apk add --no-cache qemu-system-x86

WORKDIR /game
COPY bzImage .
COPY rootfs.img .
RUN echo 'qemu-system-x86_64 -kernel bzImage -initrd rootfs.img -nographic -append "console=ttyS0"' > start.sh
RUN chmod +x start.sh

CMD ["./start.sh"]