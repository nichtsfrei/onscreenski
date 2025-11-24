# onscreenski

This is a frontend for the very unsafe backend `ukeynski` in ... you know: `../backend/`

It has layer support.


## Dependencies

Besides the dependencies mentioned in the Cargo definition you need to have:
- 📦gtk4-layer-shell-devel
insalled when building and 
- gtk4-layer-shell
- a running instance of [onscreenski](../backend/README.md)
when running.

Mainly because I didn't figure out how to tell a window-manager that the application should NOT be focused on an other framework and I stole the initial stuff from: 

- https://github.com/ptazithos/wkeys

but wanted to have a split on the left and on the right for larger tablets. 
Additionally I prefered a daemon that `uinput` instead of wayland protocol.

However if you're sane person you should probably use wkeys instead.

Also I didn't bother with error handling. unwrap all the way.

## build

It's a rust project.

## License

This repository is licensed under the [MIT License](LICENSE).
