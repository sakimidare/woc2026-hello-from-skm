// SPDX-License-Identifier: GPL-2.0
// Simple Tetris game player for the kernel tetris module

#include <fcntl.h>
#include <stdio.h>
#include <termios.h>
#include <unistd.h>

#define TETRIS_DEV "/dev/tetris"
#define BUFFER_SIZE 4096

void clear_screen() {
  printf("\033[2J\033[H");
  fflush(stdout);
}

void set_nonblocking_mode(struct termios *old_tio) {
  struct termios new_tio;
  tcgetattr(STDIN_FILENO, old_tio);
  new_tio = *old_tio;
  new_tio.c_lflag &= ~(ICANON | ECHO);
  new_tio.c_cc[VMIN] = 0;
  new_tio.c_cc[VTIME] = 0;
  tcsetattr(STDIN_FILENO, TCSANOW, &new_tio);
}

int main() {
  int fd = open(TETRIS_DEV, O_RDWR);
  if (fd < 0) {
    perror("Failed to open /dev/tetris");
    printf("\nPlease load the kernel module first:\n");
    printf("  sudo insmod src/woc2026_hello_from_skm.ko\n\n");
    return 1;
  }

  struct termios old_tio;
  set_nonblocking_mode(&old_tio);

  printf("=== Kernel Tetris Game ===\n\n");
  printf("Controls:\n");
  printf("  a/A - Move left\n");
  printf("  d/D - Move right\n");
  printf("  s/S - Move down\n");
  printf("  w/W - Rotate\n");
  printf("  Space - Hard drop\n");
  printf("  r/R - Reset game\n");
  printf("  q/Q - Quit\n\n");
  printf("Press any key to start...\n");

  getchar();

  char buffer[BUFFER_SIZE];
  char cmd;
  int auto_drop_counter = 0;
  const int auto_drop_interval = 5; // Auto drop every 5 frames

  while (1) {
    clear_screen();

    // Read game state from kernel
    ssize_t bytes = read(fd, buffer, sizeof(buffer) - 1);
    if (bytes > 0) {
      buffer[bytes] = '\0';
      printf("%s", buffer);
    } else if (bytes < 0) {
      perror("Read error");
      break;
    }

    printf("\nPress 'q' to quit\n");

    // Check for user input (non-blocking)
    if (read(STDIN_FILENO, &cmd, 1) > 0) {
      if (cmd == 'q' || cmd == 'Q') {
        break;
      }
      // Send command to kernel
      write(fd, &cmd, 1);
    }

    // Auto drop
    auto_drop_counter++;
    if (auto_drop_counter >= auto_drop_interval) {
      auto_drop_counter = 0;
      cmd = 's'; // Move down
      write(fd, &cmd, 1);
    }

    usleep(100000); // 100ms delay
  }

  // Restore terminal settings
  tcsetattr(STDIN_FILENO, TCSANOW, &old_tio);
  close(fd);

  clear_screen();
  return 0;
}
