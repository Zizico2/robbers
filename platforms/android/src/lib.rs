//! NativeActivity entry point; no gameplay or UI forks live in this crate.
#[cfg(target_os = "android")]
#[bevy::prelude::bevy_main]
fn main() {
    robbers_app::run();
}
