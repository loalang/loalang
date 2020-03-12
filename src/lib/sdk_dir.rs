use std::env::{current_dir, var};
use std::path::{Path, PathBuf};

const LOA_SDK: &str = "LOA_SDK";
static mut OVERRIDE_LOA_SDK: Option<PathBuf> = None;

pub fn override_sdk_dir<P: Into<PathBuf>>(path: P) {
    unsafe {
        OVERRIDE_LOA_SDK = Some(path.into());
    }
}

pub fn sdk_dir() -> PathBuf {
    unsafe { OVERRIDE_LOA_SDK.clone() }
        .or_else(|| var(LOA_SDK).ok().map(PathBuf::from))
        .or_else(|| current_dir().ok())
        .expect("Please set the LOA_SDK environment variable")
}

pub fn sdk_path<S: AsRef<Path>, I: IntoIterator<Item=S>>(segments: I) -> PathBuf {
    let mut path = sdk_dir();
    path.extend(segments);
    path
}

pub fn sdk_glob(segments: &[&str]) -> String {
    let sdk = sdk_dir();
    let mut path = vec![sdk.to_str().unwrap()];
    path.extend(segments);
    path.join(std::path::MAIN_SEPARATOR.to_string().as_ref())
}
