use std::process::ExitCode;

use libdave::{max_supported_protocol_version, Codec, Decryptor, Encryptor, MediaType, Session};

const USAGE: &str = "\
dave-cli - libdave self-contained build

USAGE:
    dave-cli version
    dave-cli selftest
    dave-cli keypackage <user_id> <group_id>
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["version"] => {
            version();
            Ok(())
        }
        ["selftest"] => selftest(),
        ["keypackage", user_id, group_id] => keypackage(user_id, group_id),
        _ => {
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn version() {
    println!("dave-cli {}", env!("CARGO_PKG_VERSION"));
    println!(
        "platform: {}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!(
        "max DAVE protocol version: {}",
        max_supported_protocol_version()
    );
}

fn keypackage(user_id: &str, group_id: &str) -> Result<(), String> {
    let group_id: u64 = group_id
        .parse()
        .map_err(|_| "group_id must be a u64".to_string())?;
    let mut session = Session::new(None, None).map_err(|e| e.to_string())?;
    session
        .init(max_supported_protocol_version(), group_id, user_id)
        .map_err(|e| e.to_string())?;
    let hex: String = session
        .marshalled_key_package()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    println!("{hex}");
    Ok(())
}

fn selftest() -> Result<(), String> {
    let max = max_supported_protocol_version();
    check(max >= 1, "protocol version >= 1")?;

    let mut session = Session::new(None, None).map_err(|e| e.to_string())?;
    session.init(max, 1234, "42").map_err(|e| e.to_string())?;
    check(
        !session.marshalled_key_package().is_empty(),
        "key package generated",
    )?;

    let mut enc = Encryptor::new().map_err(|e| e.to_string())?;
    enc.set_passthrough_mode(true);
    enc.assign_ssrc_to_codec(1, Codec::Opus);
    let frame = [0xF8, 0xFF, 0xFE, 1, 2, 3];
    let wire = enc
        .encrypt(MediaType::Audio, 1, &frame)
        .map_err(|e| e.to_string())?;
    let mut dec = Decryptor::new().map_err(|e| e.to_string())?;
    dec.transition_to_passthrough_mode(true);
    let plain = dec
        .decrypt(MediaType::Audio, &wire)
        .map_err(|e| e.to_string())?;
    check(plain == frame, "passthrough frame round trip")?;

    println!(
        "selftest ok (DAVE protocol v{max}, {}-{})",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    Ok(())
}

fn check(ok: bool, what: &str) -> Result<(), String> {
    if ok {
        println!("ok   {what}");
        Ok(())
    } else {
        Err(format!("FAILED: {what}"))
    }
}
