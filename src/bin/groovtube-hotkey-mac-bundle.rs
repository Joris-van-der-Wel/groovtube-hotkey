use std::env;
use std::fs::{copy, create_dir, File};
use std::path::PathBuf;
use std::io::Write;
use std::process::Command;
use clap::{Parser};
use log::info;
use groovtube_hotkey::init_logging;

const INFO_PLIST: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/Info.plist"));

#[derive(Parser, Debug)]
#[command(author, version)]
#[command(about = "Generates a macOS application bundle for GroovTubeHotkey.\n\nExample: ./target/release/generate-mac-bundle target/mac target/release/groovtube-hotkey resources", long_about = None)]
struct Args {
    /// Create a directory at this path to store output in
    output_path: PathBuf,

    /// Path to the groovtube-hotkey Mach-O binary
    binary_path: PathBuf,

    /// Path to the resources directory, containing icons, entitlements, etc
    resources_path: PathBuf,

    /// Sign the generated application bundle with the given certificate name from your macOS keychain.
    #[arg(long)]
    sign: Option<String>,

    /// Notarize the generated application bundle using the given profile from your macOS keychain.
    /// Use `xcrun notarytool store-credentials` to generate the profile.
    #[arg(long)]
    notarize: Option<String>,
}

trait CommandRun {
    fn run(&mut self, check_status: bool);
}

impl CommandRun for Command {
    fn run(&mut self, check_status: bool) {
        info!("{:?}", self);

        let status = self
            .spawn().expect("Command failed to spawn")
            .wait().expect("Command failed to wait");

        if check_status {
            assert!(status.success(), "Command failed: {}", status);
        }
    }
}

fn main()  {
    init_logging();
    let args = Args::parse();
    let icon_path = args.resources_path.join("icon.icns");
    let entitlements_path = args.resources_path.join("groovtube-hotkey.entitlements");

    let dmg_src_path = args.output_path.join("GroovTubeHotkey");
    let dmg_path = args.output_path.join("GroovTubeHotkey.dmg");
    let app_path = dmg_src_path.join("GroovTubeHotkey.app");
    let contents_path = app_path.join("Contents");
    let contents_macos_path = contents_path.join("MacOS");
    let contents_resources_path = contents_path.join("Resources");
    let info_plist_path = contents_path.join("Info.plist");

    create_dir(&args.output_path).expect("Failed to create output directory");
    create_dir(&dmg_src_path).expect("Failed to create dmg source directory");
    create_dir(&app_path).expect("Failed to create GroovTubeHotkey.app directory");
    create_dir(&contents_path).expect("Failed to create Contents directory");
    create_dir(&contents_macos_path).expect("Failed to create MacOS directory");
    create_dir(&contents_resources_path).expect("Failed to create Resources directory");

    Command::new("/bin/ln")
        .arg("-s").arg("/Applications")
        .arg(&dmg_src_path.join("Applications"))
        .run(true);

    let mut info_plist = File::create(&info_plist_path).expect("Failed to create Info.plist");
    info_plist.write_all(INFO_PLIST).expect("Failed to write Info.plist");

    copy(args.binary_path, contents_macos_path.join("groovtube")).expect("Failed to copy main executable");
    copy(icon_path, contents_resources_path.join("icon.icns")).expect("Failed to copy icon.icns");

    info!("GroovTubeHotkey.app has been created");

    let mut codesign = Command::new("/usr/bin/codesign");
    match &args.sign {
        None => codesign
            .arg("--sign").arg("-"), // adhoc
        Some(cert) => codesign
            .arg("--sign").arg(cert)
            .arg("--timestamp"), // adhoc does not work using apple's timestamp authority server
    };
    codesign
        .arg("--options").arg("runtime")
        .arg("--entitlements").arg(entitlements_path)
        .arg(&app_path)
        .run(true);
    info!("GroovTubeHotkey.app has been signed");

    Command::new("/usr/bin/hdiutil")
        .arg("create")
        .arg("-srcFolder").arg(dmg_src_path)
        .arg("-o").arg(&dmg_path)
        .run(true);
    info!("GroovTubeHotkey.dmg has been created");

    let mut codesign = Command::new("/usr/bin/codesign");
    match &args.sign {
        None => codesign
            .arg("--sign").arg("-"),
        Some(cert) => codesign
            .arg("--sign").arg(cert)
            .arg("--timestamp"),
    };
    codesign
        .arg("-i").arg("nl.groovtube.groovtube-hotkey")
        .arg(&dmg_path)
        .run(true);
    info!("GroovTubeHotkey.dmg has been signed");

    if let Some(profile) = args.notarize {
        Command::new("/usr/bin/xcrun")
            .arg("notarytool")
            .arg("submit")
            .arg(&dmg_path)
            .arg("--keychain-profile")
            .arg(&profile)
            .arg("--wait")
            .run(true);
        info!("GroovTubeHotkey.dmg has been submitted to apple's notary service.");
        info!("To review any issues run: xcrun notarytool log <UUID> --keychain-profile '{}'", profile);

        Command::new("/usr/bin/xcrun")
            .arg("stapler")
            .arg("staple")
            .arg(&dmg_path)
            .run(true);
        info!("GroovTubeHotkey.dmg has been stapled.");
    }

    info!("\nVerification result:");

    Command::new("/usr/bin/codesign")
        .arg("--verify")
        .arg("--deep")
        .arg("--strict")
        .arg("--verbose=2")
        .arg(&app_path)
        .run(false);

    Command::new("/usr/sbin/spctl")
        .arg("-a")
        .arg("-t").arg("open")
        .arg("--context").arg("context:primary-signature")
        .arg("-vv")
        .arg(&dmg_path)
        .run(false);

    info!("\nAll done!");
}

// TODO: The Info.plist file seems to be ignored in the menu that opens after clicking the
// application name in the Mac menu bar. Instead, the name of the binary is used.
// I am not sure what the correct fix for this is, however it appears to be something
// that is missing from (or added by accident to) the binary.
