import { describe, expect, it } from 'vitest';
import {
  Codec,
  DaveSessionManager,
  Decryptor,
  Encryptor,
  MediaType,
  maxSupportedProtocolVersion,
  Session,
  setLogSink,
} from '../src/index.js';

const GROUP = 1234567890123456789n;
const FRAME = Buffer.from([0xf8, 0xff, 0xfe, 1, 2, 3, 4]);

describe('library', () => {
  it('reports a protocol version', () => {
    expect(maxSupportedProtocolVersion()).toBeGreaterThanOrEqual(1);
  });

  it('accepts and removes a log sink', () => {
    expect(() => setLogSink(() => {})).not.toThrow();
    expect(() => setLogSink(null)).not.toThrow();
  });
});

describe('Session', () => {
  it('produces a key package', () => {
    const session = new Session();
    session.init(maxSupportedProtocolVersion(), GROUP, '42');
    expect(session.getMarshalledKeyPackage().length).toBeGreaterThan(0);
    session.destroy();
  });

  it('rejects an invalid group id', () => {
    const session = new Session();
    expect(() => session.init(1, 'nope', '42')).toThrow(/groupId/);
    expect(() => session.init(1, -1n, '42')).toThrow(/groupId/);
    expect(() => session.init(1, 2n ** 64n, '42')).toThrow(/groupId/);
  });

  it('survives garbage MLS input', () => {
    const session = new Session();
    session.init(maxSupportedProtocolVersion(), GROUP, '42');
    const junk = Buffer.from([1, 2, 3]);
    expect(session.processCommit(junk).roster).toEqual([]);
    expect(session.processWelcome(junk)).toBeNull();
    expect(session.processProposals(junk)).toBeNull();
    expect(Array.isArray(session.takeFailures())).toBe(true);
  });

  it('tracks recognized users', () => {
    const session = new Session();
    session.addRecognizedUser('43');
    session.removeRecognizedUser('43');
    expect(() => session.addRecognizedUser('4\u00003')).toThrow(/user id/);
  });

  it('rejects a fingerprint request without an MLS group', async () => {
    const session = new Session();
    session.init(maxSupportedProtocolVersion(), GROUP, '42');
    await expect(session.getPairwiseFingerprint(0, '43')).rejects.toThrow(
      /fingerprint/,
    );
  });

  it('throws after destroy', () => {
    const session = new Session();
    session.destroy();
    session.destroy();
    expect(() => session.getMarshalledKeyPackage()).toThrow(/destroyed/);
  });
});

describe('Encryptor and Decryptor', () => {
  it('round-trips in passthrough mode', () => {
    const encryptor = new Encryptor();
    encryptor.setPassthroughMode(true);
    encryptor.assignSsrcToCodec(1, Codec.Opus);
    expect(encryptor.passthroughMode).toBe(true);
    const wire = encryptor.encrypt(MediaType.Audio, 1, FRAME);

    const decryptor = new Decryptor();
    decryptor.transitionToPassthroughMode(true);
    expect(decryptor.decrypt(MediaType.Audio, wire)).toEqual(FRAME);
  });

  it('throws when encrypting without keys', () => {
    const encryptor = new Encryptor();
    encryptor.assignSsrcToCodec(7, Codec.Opus);
    expect(() => encryptor.encrypt(MediaType.Audio, 7, FRAME)).toThrow(
      /encryption failed/i,
    );
  });
});

describe('DaveSessionManager', () => {
  it('drives a downgrade transition', () => {
    const manager = new DaveSessionManager('42', '99');
    manager.addUser('43');
    manager.onSelectProtocolAck(maxSupportedProtocolVersion());
    expect(manager.getMarshalledKeyPackage().length).toBeGreaterThan(0);

    expect(manager.onPrepareTransition(5, 0)).toBe(5);
    manager.onExecuteTransition(5);
    expect(manager.encrypt(MediaType.Audio, 1, FRAME)).toEqual(FRAME);
    expect(manager.decrypt(MediaType.Audio, '43', FRAME)).toEqual(FRAME);
    expect(manager.getDecryptStats('43', MediaType.Audio)).not.toBeNull();
    expect(manager.getDecryptStats('nobody', MediaType.Audio)).toBeNull();
    manager.destroy();
    expect(() => manager.addUser('44')).toThrow(/destroyed/);
  });
});
