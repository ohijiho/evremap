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

## Configuration

Here's an example configuration that makes capslock useful:

```toml
# The name of the device to remap.
# Run `sudo evremap list-devices` to see the devices available
# on your system.
device_name = "AT Translated Set 2 keyboard"

# Configure CAPSLOCK as a Dual Role key.
# Holding it produces LEFTCTRL, but tapping it
# will produce ESC.
# Both `tap` and `hold` can expand to multiple output keys.
[[dual_role]]
input = "KEY_CAPSLOCK"
hold = ["KEY_LEFTCTRL"]
tap = ["KEY_ESC"]
```

You can also express simple remapping entries:

```
# This config snippet is useful if your keyboard has an arrow
# cluster, but doesn't have page up, page down, home or end
# keys.  Here we're configuring ALT+arrow to map to those functions.
[[remap]]
input = ["KEY_LEFTALT", "KEY_UP"]
output = ["KEY_PAGEUP"]

[[remap]]
input = ["KEY_LEFTALT", "KEY_DOWN"]
output = ["KEY_PAGEDOWN"]

[[remap]]
input = ["KEY_LEFTALT", "KEY_LEFT"]
output = ["KEY_HOME"]

[[remap]]
input = ["KEY_LEFTALT", "KEY_RIGHT"]
output = ["KEY_END"]
```

When applying remapping configuration, ordering is important:

* Dual Role entries are always processed first
* Remap entries are applied in the order that they appear in
  your configuration file

Here's an example where ordering is important: on the PixelBook Go keyboard,
the function key row has alternate functions on the keycaps.  It is natural
to want the mute button to mute by default, but to emit the F8 key when
holding alt.  We can express that with the following configuration:

```
[[remap]]
input = ["KEY_LEFTALT", "KEY_F8"]
# When our `input` is matched, our list of `output` is prevented from
# matching as the `input` of subsequent rules.
output = ["KEY_F8"]

[[remap]]
input = ["KEY_F8"]
output = ["KEY_MUTE"]
```

## Building it

```
$ sudo dnf install libevdev-devel
$ cargo build --release
```

## Running it

To run the remapper, invoke it *as root* (so that it can grab exclusive access to the input device):

```
sudo target/release/evremap remap my-config-file.toml
```

## Systemd

A sample system service unit is included in the repo.  You'll want to adjust the paths to match
your system and then install and enable it:

```
$ sudo cp evremap.service /usr/lib/systemd/system/
$ sudo systemctl enable evremap.service
$ sudo systemctl start evremap.service
```
