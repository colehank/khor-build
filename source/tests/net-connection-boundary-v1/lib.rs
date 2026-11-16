#![cfg(target_os = "linux")]
#[cfg(test)]
mod silent;
#[cfg(test)]
mod upstream_util;
#[cfg(test)]
#[ctor::ctor(unsafe)]
fn userns() {
    patchbay::init_userns().expect("isolated user/network namespaces");
}
