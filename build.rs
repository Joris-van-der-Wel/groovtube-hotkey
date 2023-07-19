extern crate image;
use std::{env, fs};
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;

fn package_dir() -> String {
    env::var("CARGO_MANIFEST_DIR").expect("No CARGO_MANIFEST_DIR env var")
}

fn out_dir() -> String {
    env::var("OUT_DIR").expect("No OUT_DIR env var")
}

fn build_window_icon() {
    let out_dir = out_dir();
    let img_path: PathBuf = [package_dir().as_str(), "resources", "icon-32.png"].iter().collect();
    let out_path: PathBuf = [out_dir.as_str(), "icon-32-rgba"].iter().collect();

    let img = image::open(img_path).expect("Failed to read/decode icon-32-rgba");
    let img = img.to_rgba8();
    let rgba = img.into_raw();
    println!("DEBUG: writing window icon to {}", out_path.to_str().unwrap());
    fs::write(&out_path, rgba).expect("Failed to write to microswitch-icon-32-rgba");
}

fn make_info_plist() {
    let out_dir = out_dir();
    let out_path: PathBuf = [out_dir.as_str(), "Info.plist"].iter().collect();
    let mut output = File::create(out_path).expect("Failed to create Info.plist");
    let version = env!("CARGO_PKG_VERSION");

    // Should escape dynamic values here, however at the moment only the version string is dynamic.
    // So we can get away with not escaping for now.

    // CFBundleName should be max 15 characters

    write!(output, r##"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>

	<key>NSPrincipalClass</key>
	<string>NSApplication</string>

	<key>CFBundlePackageType</key>
	<string>APPL</string>

	<key>CFBundleIdentifier</key>
	<string>nl.groovtube.groovtube-hotkey</string>

	<key>CFBundleName</key>
	<string>GroovTubeHotkey</string>

	<key>CFBundleDisplayName</key>
	<string>GroovTube Hotkey</string>

	<key>CFBundleVersion</key>
	<string>{version}</string>

	<key>CFBundleShortVersionString</key>
	<string>{version}</string>

	<key>NSHumanReadableCopyright</key>
	<string>Copyright (c) 2023 Joris van der Wel</string>

	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>

	<key>LSApplicationCategoryType</key>
	<string>public.app-category.utilities</string>

	<key>CFBundleExecutable</key>
	<string>groovtube</string>

	<key>CFBundleSupportedPlatforms</key>
	<array>
		<string>MacOSX</string>
	</array>

	<key>NSBluetoothAlwaysUsageDescription</key>
	<string>GroovTube Hotkey wants to automatically connect to the GroovTube peripheral</string>

	<key>NSHighResolutionCapable</key>
	<true/>

	<key>NSSupportsSuddenTermination</key>
	<true/>

	<key>CFBundleIconFile</key>
	<string>icon.icns</string>

</dict>
</plist>
"##, version=version).expect("Failed to write to Info.plist");
}

fn common_steps() {
    make_info_plist();
    build_window_icon();
}

#[cfg(target_os = "windows")]
fn main() {
    let package_dir = package_dir();
    let resources_dir: PathBuf = [package_dir.as_str(), "resources"].iter().collect();

    println!("cargo:rustc-link-search=native={}", resources_dir.to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib={}", "resources");

    common_steps()
}

#[cfg(target_os = "macos")]
fn main() {
    common_steps();
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn main() {
    common_steps();
}
