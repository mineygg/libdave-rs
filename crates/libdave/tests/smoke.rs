use libdave::{
    max_supported_protocol_version, Codec, Decryptor, Encryptor, MediaType, Session, SessionManager,
};

#[test]
fn reports_a_protocol_version() {
    assert!(max_supported_protocol_version() >= 1);
}

#[test]
fn session_produces_a_key_package() {
    let mut session = Session::new(None, None).expect("session");
    session
        .init(max_supported_protocol_version(), 1_234_567_890, "42")
        .expect("init");
    let kp = session.marshalled_key_package();
    assert!(!kp.is_empty(), "key package should not be empty");
}

#[test]
fn passthrough_round_trip() {
    let mut enc = Encryptor::new().expect("encryptor");
    enc.set_passthrough_mode(true);
    enc.assign_ssrc_to_codec(1, Codec::Opus);
    assert!(enc.is_passthrough_mode());

    let frame = [0xF8u8, 0xFF, 0xFE, 1, 2, 3, 4];
    let wire = enc.encrypt(MediaType::Audio, 1, &frame).expect("encrypt");
    assert_eq!(wire, frame);

    let mut dec = Decryptor::new().expect("decryptor");
    dec.transition_to_passthrough_mode(true);
    let plain = dec.decrypt(MediaType::Audio, &wire).expect("decrypt");
    assert_eq!(plain, frame);
}

#[test]
fn encrypting_without_keys_fails_cleanly() {
    let mut enc = Encryptor::new().expect("encryptor");
    enc.assign_ssrc_to_codec(7, Codec::Opus);
    assert!(enc.encrypt(MediaType::Audio, 7, &[1, 2, 3, 4]).is_err());
}

#[test]
fn manager_drives_a_session() {
    let mut mgr = SessionManager::new("42", 99, None).expect("manager");
    mgr.add_user("43").expect("add user");
    mgr.on_select_protocol_ack(max_supported_protocol_version())
        .expect("ack");
    assert!(!mgr.marshalled_key_package().is_empty());

    // Downgrade to the unencrypted protocol: everything must go through in passthrough.
    assert_eq!(mgr.on_prepare_transition(5, 0).expect("prepare"), Some(5));
    mgr.on_execute_transition(5).expect("execute");
    let frame = [9u8, 8, 7, 6];
    assert_eq!(
        mgr.encrypt(MediaType::Audio, 1, &frame).expect("encrypt"),
        frame
    );
}
