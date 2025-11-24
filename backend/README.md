# ukeynski

ukeynski is a small Linux utility that creates a virtual keyboard using the uinput kernel interface and listens on a UNIX domain socket for incoming key codes.
Each received byte is interpreted as a Linux input key code and injected into the system as a keyboard event.

This allows external programs, scripts, or remote processes to simulate keyboard key presses on the host machine by sending raw key codes.


The protocol is very simple: one byte on keycode.

This is possible because it only registers:
```c
  for (i = 1; i < 249; ++i)
    ioctl(fd, UI_SET_KEYBIT, i);

```


## Socket Path

The socket is created at:

`$XDG_RUNTIME_DIR/ukeynski.socket`

If `XDG_RUNTIME_DIR` is not set, it falls back to:

`/tmp/ukeynski.socket`

## Build

```bash
	gcc -o ukeynski ukeynski.c
```

## Sending Key Codes

The keycodes can be looked up at:
- input_event_codes.h

or in

- ../frontend/src/ui/supported_keys.rs

To simply test via nc:

```bash
printf "\x2A\x23\x2A\x12\x26\x26\x18\x39\x11\x18\x13\x26\x20" | nc -U "$XDG_RUNTIME_DIR/ukeynski.socket"
```
