**The App**
[app look](./asset/)

**Known issues**
1. winit/WSL window::resize_events() - iced apps on WSL2's Wayland compositor hit broken-pipe IO errors, eventually losing the Wayland socket entirely, and separately WSLg's Weston compositor doesn't fully support native window-management features like resize/maximize. 
WSLg also runs an XWayland server alongside Weston. `WAYLAND_DISPLAY= DISPLAY=:0 cargo run` force winit's platform detection falls back to X11 via XWayland.
2. window size - triggering the above issue as well. a larger surface means a bigger shared-memory buffer has to cross the Wayland socket between winit and Weston inside WSLg.
3. 
