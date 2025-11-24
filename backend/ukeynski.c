#include <fcntl.h>
#include <linux/input-event-codes.h>
#include <linux/uinput.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/select.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/un.h>
#include <unistd.h>

#define SOCKET_PATH_CAP 512
#define BUFFER_SIZE 1024
#define ARRAY_LEN(arr) (sizeof(arr)/sizeof(arr[0]))

void emit(int fd, int type, int code, int val) {
  struct input_event ie = {0};

  ie.type = type;
  ie.code = code;
  ie.value = val;

  write(fd, &ie, sizeof(ie));
}

void press(int fd, int code) {
  emit(fd, EV_KEY, code, 1);
  emit(fd, EV_SYN, SYN_REPORT, 0);
}

void release(int fd, int code) {
  emit(fd, EV_KEY, code, 0);
  emit(fd, EV_SYN, SYN_REPORT, 0);
}

void press_and_release(int fd, int code) {
  press(fd, code);
  release(fd, code);
}

int initialize_keyboard() {
  struct uinput_setup usetup = {0};
  int fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);
  int i;

  ioctl(fd, UI_SET_EVBIT, EV_KEY);

  for (i = 1; i < 249; ++i)
    ioctl(fd, UI_SET_KEYBIT, i);

  usetup.id.bustype = BUS_USB;
  usetup.id.vendor = 0x2323;
  usetup.id.product = 0x4242;
  strcpy(usetup.name, "ukeynski");

  ioctl(fd, UI_DEV_SETUP, &usetup);
  ioctl(fd, UI_DEV_CREATE);
  return fd;
}

const char *socket_path() {
  const char *runtime_dir;
  static char socket_path[SOCKET_PATH_CAP] = {0};
  if ((runtime_dir = getenv("XDG_RUNTIME_DIR")) == NULL) {
    runtime_dir = "/tmp/";
  }
  snprintf(socket_path, SOCKET_PATH_CAP, "%s/ukeynski.socket", runtime_dir);
  return socket_path;
}

void error_and_exit(const char *indicator) {
  // TODO: set kb_fd and server_fd to global that may be handeld here
  perror(indicator);
  exit(EXIT_FAILURE);
}

int initialize_unix_socket() {
  int fd;
  struct sockaddr_un server_addr = {0};
  const char *path = socket_path();

  if ((fd = socket(AF_UNIX, SOCK_STREAM, 0)) == -1)
    error_and_exit("socket");
  server_addr.sun_family = AF_UNIX;
  strncpy(server_addr.sun_path, path, sizeof(server_addr.sun_path) - 1);
  unlink(path);
  if (bind(fd, (struct sockaddr *)&server_addr, sizeof(server_addr)) == -1)
    goto fail;
  if (listen(fd, 1) == -1)
    goto fail;

  printf("ukeynski listening on %s\n", path);

  return fd;
fail:
  close(fd);
  error_and_exit("listening");
  return -1;
}

const unsigned char mod_keys[] = {
    KEY_LEFTCTRL,  KEY_RIGHTCTRL,  KEY_LEFTMETA, KEY_RIGHTMETA,
    KEY_LEFTSHIFT, KEY_RIGHTSHIFT, KEY_LEFTALT,  KEY_RIGHTALT,
};

int is_mod_key(unsigned char kc) {
  for (ssize_t i = 0; i < ARRAY_LEN(mod_keys); ++i) {
    if (kc == mod_keys[i])
      return 1;
  }
  return 0;
}

void start_listener(int server_fd, int kb_fd) {
  char buffer[BUFFER_SIZE] = {0};
  char press_state[255] = {0};
  int activity;
  int client_fd;
  ssize_t bytes_recv;
  ssize_t i;
  while (1) {
    if ((client_fd = accept(server_fd, NULL, NULL)) == -1)
      error_and_exit("accept");
    while ((bytes_recv = read(client_fd, buffer, BUFFER_SIZE)) > 0) {
      for (i = 0; i < bytes_recv; ++i) {
        unsigned char kbn = buffer[i];
        if (kbn == 0)
          continue;
        printf("[%d]\t", kbn);
        if (is_mod_key(kbn) == 1) {
          if (press_state[kbn] == 1) {
            printf("release\n");
            release(kb_fd, kbn);
            press_state[kbn] = 0;
          } else {
            printf("press\n");
            press(kb_fd, kbn);
            press_state[kbn] = 1;
          }
        } else {
          press_and_release(kb_fd, kbn);
          // safe guard for single KC if there is is released modifier we press
          // it to not keep otherwise we assume it is on purpose
          if (bytes_recv == 1) {
            for (ssize_t j = 0; j < ARRAY_LEN(mod_keys); ++j) {
              if (press_state[mod_keys[j]] == 1) {
                printf("removing %d ", mod_keys[j]);
                release(kb_fd, mod_keys[j]);
                press_state[mod_keys[j]] = 0;
              }
            }
          }
          printf("press and release\n");
        }
      }
    }
    if (bytes_recv == -1)
      perror("recv");
    close(client_fd);
  }
}

int main(void) {
  int kb_fd = initialize_keyboard();
  int ux_fd = initialize_unix_socket();
  start_listener(ux_fd, kb_fd);
  ioctl(kb_fd, UI_DEV_DESTROY);
  close(kb_fd);
  close(ux_fd);
  return 0;
}
