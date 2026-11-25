# D03 authorized inputs

D03 follows the merged D02 foundation. The first slice combines WP01 image validation with the WP02 authorization boundary and WP06 session ownership.

| Package | Requirements | Ready and acceptance conditions |
|---|---|---|
| WP01 · Static image | FR05, FR14, FR61 | Use the local desktop file chooser. Rust accepts exactly one explicitly authorized local PNG or JPEG. Reject invalid or oversized files, show verified dimensions and report unreadable input. Exercise valid fixtures, malformed images, unsupported formats and size limits. Image interpretation belongs to D04. |
| WP02 · Native source grants | FR02–FR04, FR13, FR27–FR28, FR48–FR49, FR52, FR56–FR59, FR62 | Rust owns session-scoped grants. GNOME ScreenCast grants one selected window and supplies the PipeWire remote. Audio uses an explicitly selected permitted source. Test denial, revocation, disconnection and isolated combined-mode loss with permitted fixtures. |
| WP06 · Cleanup | FR38, FR50, FR59–FR60, ENG07, REL03 | Pause, stop, window close, lock and sleep invalidate grants and release stream handles. An obsolete request can never publish into a new session. Verify normal and interrupted teardown on the reference desktop. |

The WebView receives source status and image metadata. Native handles, paths and raw content stay in Rust. D03 keeps all inputs local, with no recording or remote processing. Desktop app status must distinguish a failed or unreadable input from a known platform restriction.

The static-image boundary uses the XDG file chooser with a single selection. App consent and an active compatible session are checked before opening it. A request identifier ties the result to the active session, and pause or stop invalidates it. File validation has provisional 16 MiB and 4,096-pixel per-axis bounds. Only decoded dimensions and format are kept for this slice.

The screen adapter asks GNOME ScreenCast for a single window with no persistent grant, then opens only the returned PipeWire remote. Rust enumerates audio sources using the PipeWire device provider and retains session-scoped opaque IDs for exact serials. The application accepts no source path or PipeWire target from the WebView.

The GStreamer pipelines hold at most one delivered sample and use bounded formats. At this milestone the application reports availability. Image, speech and visual interpretation follow in D04. Pause and stop cancel input requests. A source stays in the releasing state until its worker has returned from cleanup, so a new configuration cannot race a prior stream. The desktop client subscribes to screen lock and system sleep and withdraws grants when those signals arrive. Window minimization requests release live sources to avoid background operation without app controls.

The reference build used an unprivileged SDK extracted from the matching Ubuntu developer packages and linked the existing GStreamer 1.28.2 runtime. The production manifest pins the Rust GStreamer bindings and the Debian bundle declares the runtime plug-in.
