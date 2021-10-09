# Notes on Wayland and clipboard support

### Running espanso as another user

When running espanso as another user, we need to set up a couple of permissions
in order to enable the clipboard tools to correctly connect to the Wayland desktop.

In particular, we need to add the `espanso` user to the same group as the current user
so that it can access the `/run/user/X` directory (with X depending on the user).

```
# Find the current user wayland dir with
echo $XDG_RUNTIME_DIR  # in my case output: /run/user/1000

ls -la /run/user/1000

# Now add the `espanso` user to the current user group
sudo usermod -a -G freddy espanso

# Give permissions to the group
chmod g+rwx /run/user/1000

# Give write permission to the wayland socket
chmod g+w /run/user/1000/wayland-0
```

Now the clipboard should work as expected

## Better implementation

On some Wayland compositors (currently sway), the "wlr-data-control" protocol could enable the use of a much more efficient implementation relying on the "wl-clipboard-rs" crate.

Useful links: https://github.com/YaLTeR/wl-clipboard-rs/issues/8