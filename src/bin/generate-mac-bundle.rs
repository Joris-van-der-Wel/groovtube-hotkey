use std::env;
use std::fs::{copy, create_dir, File};
use std::path::PathBuf;
use std::io::Write;

fn make_info_plist(mut output: File, version: &str) -> Result<(), std::io::Error> {
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
	<string>GroovTubeHotkey</string>

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
"##, version=version)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <output_path> <binary_path> <icns_path>", &args[0]);
        eprintln!("Example: {} 'target/GroovTubeHotkey.app' target/release/groovtube-hotkey resources/icon.icns", &args[0]);
        return Err("Not enough / too many arguments".into());
    }

    let output_path = PathBuf::from(&args[1]);
    let binary_path = PathBuf::from(&args[2]);
    let icns_path = PathBuf::from(&args[3]);

    let contents_path = output_path.join("Contents");
    let contents_macos_path = contents_path.join("MacOs");
    let contents_resources_path = contents_path.join("Resources");
    let info_plist_path = contents_path.join("Info.plist");

    create_dir(&output_path)?;
    create_dir(&contents_path)?;
    create_dir(&contents_macos_path)?;
    create_dir(&contents_resources_path)?;

    let info_plist = File::create(&info_plist_path)?;
    make_info_plist(info_plist, env!("CARGO_PKG_VERSION"))?;

    copy(binary_path, contents_macos_path.join("GroovTubeHotkey"))?;
    copy(icns_path, contents_resources_path.join("icon.icns"))?;

    println!("Done!");
    Ok(())
}

// TODO: The Info.plist file seems to be ignored in certain places:
// - In the menu that opens after clicking the application name in the Mac menu bar.
//   Instead, the name of the binary is used.
// - Within the list of applications in "Privacy & Security" -> "Accessibility". (which
//   also shows a terminal icon instead of our own icon). However the list in
//   "Privacy & Security" -> "Bluetooth" shows the proper name and icon.
// The name of the binary is set to "GroovTubeHotkey" as a workaround.
// I am not sure what the correct fix for this is, however it appears to be something
// that is missing from (or added by accident to) the binary.
