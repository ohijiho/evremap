# evremap

*A keyboard input remapper for Linux/Wayland systems, written by <a href="https://github.com/wez/">@wez</a>*

## Why?

I couldn't find a good solution for the following:

* Remap the `CAPSLOCK` key so that it produces `CTRL` when held, but `ESC` if tapped
* Remap N keys to M keys.  Eg: `F3` -> `CTRL+c`, and `ALT+LEFT` to `HOME`

## How?

`evremap` works by grabbing exclusive access to an input device and maintaining
a model of the keys that are pressed.  It then applies your remapping configuration
to produce the effective set of pressed keys and emits appropriate changes to a virtual
output device.

Because `evremap` targets the evdev layer of libinput, its remapping
is effective system-wide: in Wayland, X11 and the linux console.

## Configuration

Here's an example configuration that enables MacOS-like meta key.

```toml
# The name of the device to remap.
# Run `sudo evremap list-devices` to see the devices available
# on your system.
device_name = "PFU Limited HHKB-Hybrid"

# If you have multiple devices with the same name, you can optionally
# specify the `phys` value that is printed by the `list-devices` subcommand
# phys = "usb-0000:07:00.3-2.1.1/input0"

[[remap]]
cond = []
except = []
when = ["KEY_LEFTMETA"]
[remap.mappings]
KEY_LEFTMETA = ["KEY_LEFTCTRL"]

[[remap]]
cond = []
except = []
when = ["KEY_TAB", "KEY_GRAVE"]
[remap.mappings]
KEY_LEFTMETA = ["KEY_LEFTMETA"]
```

Python script can be useful for generating complex configuration files. An example can be found here: `scripts/example1.py`.

* How do I list available input devices?
  `sudo evremap list-devices`

* How do I list available key codes?
  `evremap list-keys`

## Building it

```console
$ sudo dnf install libevdev-devel # redhat/centos
## or
$ sudo apt install libevdev-dev pkg-config # debian/ubuntu

$ cargo build --release
```

## Running it

To run the remapper, invoke it *as root* (so that it can grab exclusive access to the input device):

```console
$ sudo target/release/evremap remap my-config-file.toml
```

Or, grant an unprivileged user access to `evdev` and `uinput`.
On Ubuntu, this can be configured by running the following commands and rebooting:

```
sudo gpasswd -a YOUR_USER input
echo 'KERNEL=="uinput", GROUP="input"' | sudo tee /etc/udev/rules.d/input.rules
```

For some platforms, you might need to create an `input` group first and run:
```
echo 'KERNEL=="event*", NAME="input/%k", MODE="660", GROUP="input"' | sudo tee /etc/udev/rules.d/input.rules
```
as well.

## Systemd

A sample system service unit is included in the repo.  You'll want to adjust the paths to match
your system and then install and enable it:

```console
$ sudo cp evremap.service /usr/lib/systemd/system/
$ sudo systemctl daemon-reload
$ sudo systemctl enable evremap.service
$ sudo systemctl start evremap.service
```

## Runit

If you're using Runit instead of Systemd, follow these steps to create a service.

* Create a directory called `evremap` and create a file called `run` under it
```console
sudo mkdir /etc/sv/evremap
sudo touch /etc/sv/evremap/run
```

* Copy these lines into the run file
```console
#!/bin/sh
set -e 
exec <PATH_TO_EVREMAP> remap <CONFIG>
```

Replace `<PATH_TO_EVREMAP>` with the path to your evremap executable and `<CONFIG>` with the path to your configuration file.

* Finally, symlink the evremap directory to `/var/service`
```console
sudo ln -s /etc/sv/evremap /var/service
```

## OpenRC

To make an OpenRC service, create the file `/etc/init.d/evremap` with the following contents...

```console
#!/usr/bin/openrc-run

supervisor=supervise-daemon
command="<PATH_TO_EVREMAP>"
command_args="remap <CONFIG>"
```

Replace `<PATH_TO_EVREMAP>` with the path to your evremap executable and `<CONFIG>` with the path to your configuration file.

Make the file executable...

```console
chmod +x /etc/init.d/evremap
```

Enable the service with...

```console
rc-update add evremap
```

Start the service with...

```console
rc-service evremap start
```

## How do I make this execute a command when a key is pressed?

That feature is not implemented.
