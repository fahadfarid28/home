// built by build.rs
#[link(name = "rubicon_exports", kind = "dylib")]
unsafe extern "C" {
    unsafe fn never_fear_rubicon_exports_is_here();
}

pub fn hi() {
    unsafe {
        never_fear_rubicon_exports_is_here();
    }
}
