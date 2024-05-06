use std::env;
use std::fs::{copy, create_dir, File};
use std::path::PathBuf;
use std::io::Write;
use std::process::{Command, Output};
use clap::{Parser};
use log::info;
use x509_parser::pem::Pem;
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
    fn run_capture_output(&mut self, check_status: bool) -> Output;
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

    fn run_capture_output(&mut self, check_status: bool) -> Output {
        info!("{:?}", self);

        let output = self
            .output().expect("Command failed to spawn");

        if check_status {
            assert!(output.status.success(), "Command failed: {}", output.status);
        }

        return output;
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

    // (the team ID)
    let cert_subject_ou: Option<String> = match &args.sign {
        None => None,
        Some(cert) => {
            let output = Command::new("/usr/bin/security")
                .arg("find-certificate")
                .arg("-c").arg(cert)
                .arg("-p")
                .run_capture_output(true);

            let mut result: Option<String> = None;

            for pem in Pem::iter_from_buffer(&output.stdout) {
                let pem = pem.expect("Failed to decode PEM certificate");
                let x509 = pem.parse_x509().expect("Failed to decode DER certificate");
                for ou in x509.subject.iter_organizational_unit() {
                    result = Some(
                        ou.as_str().expect("Failed to decode certificate subject OU")
                            .to_string()
                    );
                }
            }

            result
        },
    };

    let mut codesign = Command::new("/usr/bin/codesign");
    match &args.sign {
        None => codesign
            .arg("--sign").arg("-"), // adhoc
        Some(cert) => codesign
            .arg("--sign").arg(cert)
            .arg("--requirements")
            .arg(format!(
                "=designated => identifier \"nl.groovtube.groovtube-hotkey\" and \
                anchor apple generic and \
                certificate 1[field.1.2.840.113635.100.6.2.6] /* exists */ and \
                certificate leaf[field.1.2.840.113635.100.6.1.13] /* exists */ and \
                certificate leaf[subject.OU] = \"{}\"",
                cert_subject_ou.expect("Could not find certificate subject OU")
            ))
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

    // This command should output the line "satisfies its Designated Requirement".
    // If not, the app will launch, however it will not be possible to access bluetooth and
    // accessibility. System Setting will list that the app has access, however the access will
    // not be applied because The Designated Requirement is incorrect.
    // There are several things to check:
    //
    // Access is stored in Apple's TCC sqlite database. The Designated Requirement is stored in
    // the "csreq" column in the table "access" in "TCC.db".
    // This database can be found at:
    //   /Library/Application\ Support/com.apple.TCC/TCC.db
    //   ~/Library/Application\ Support/com.apple.TCC/TCC.db
    // And can be accessed by temporarily giving the terminal Full Disk Access. Example query:
    //   sqlite3 /Library/Application\ Support/com.apple.TCC/TCC.db \
    //   'select service, client, auth_value, auth_reason, HEX(csreq) from access where client = "nl.groovtube.groovtube-hotkey"'
    // Or, to dump the entire database:
    //   sqlite3 /Library/Application\ Support/com.apple.TCC/TCC.db .dump
    //
    // This app requires the "service": "kTCCServiceBluetoothAlways" (user TCC.cb) and "kTCCServiceAccessibility" (global TCC.db).
    // "auth_value" is one of: denied(0), unknown(1), allowed(2), or limited(3).
    // The "csreq" can be decoded as follows:
    //   echo 'fade0c...' | xxd -r -p | csreq -r- -t
    //
    // The Designated Requirement of the app can be extracted as follows:
    //  ` codesign -d -r-` GroovTubeHotkey.app
    // This will look like:
    //   designated => identifier "nl.groovtube.groovtube-hotkey" and certificate leaf = H"85022bd9cff596d57c094a8265e7798fd3d9a201"
    // For syntax see:
    //   https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/RequirementLang/RequirementLang.html
    //
    // The certificates that were used to sign the app can be extracted as follows:
    //   codesign -dvvv --extract-certificates GroovTubeHotkey.app
    // This will place codesign0, codesign1, codesign2 in the current directory.
    // "certificate leaf" refers to codesign0, "certificate root" refers to the highest number (codesign2)
    // The SHA1 fingerprint can be extracted as follows:
    //   openssl x509 -inform DER -in codesign0 -text -fingerprint
    // This should correspond to the hex value of the "certificate leaf" requirement statement
    //
    // It is possible to test for requirements manually using codesign -R='certificate leaf = H"123..."'

    Command::new("/usr/bin/codesign")
        .arg("--verify")
        .arg("--requirements").arg("-")
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
