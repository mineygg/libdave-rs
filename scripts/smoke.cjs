// Loads one prebuilt binary directly: `node scripts/smoke.cjs ./libdave.<platform>.node`
const assert = require('node:assert');
const { resolve } = require('node:path');

const binding = require(resolve(process.argv[2]));
assert(binding.maxSupportedProtocolVersion() >= 1);

const session = new binding.Session();
session.init(binding.maxSupportedProtocolVersion(), '99', '42');
assert(session.getMarshalledKeyPackage().length > 0);

const encryptor = new binding.Encryptor();
encryptor.setPassthroughMode(true);
const frame = Buffer.from([1, 2, 3, 4]);
assert.deepStrictEqual(
  encryptor.encrypt(binding.MediaType.Audio, 1, frame),
  frame,
);
console.log(
  `smoke ok: ${process.argv[2]} (${process.platform}-${process.arch})`,
);
