use std::env;
use std::fs::{copy, create_dir, File};
use std::path::PathBuf;
use std::io::Write;

const INFO_PLIST: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Info.plist"));

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

    let mut info_plist = File::create(&info_plist_path)?;
    info_plist.write_all(INFO_PLIST)?;

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
