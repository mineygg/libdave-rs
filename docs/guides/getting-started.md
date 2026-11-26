# Getting started

## Requirements

- Node.js 18 or newer.
- A supported platform binary distributed through the package's optional platform dependencies.

The package ships prebuilt native bindings. A normal npm install does not require a local Rust or C++ toolchain.

## Install

```sh
npm install @mineygg/libdave-rs
```

## ESM

```ts
import {
  DaveSessionManager,
  MediaType,
  maxSupportedProtocolVersion,
} from "@mineygg/libdave-rs";

const manager = new DaveSessionManager(selfUserId, channelId);

manager.addUser(otherUserId);
manager.onSelectProtocolAck(maxSupportedProtocolVersion());

const keyPackage = manager.getMarshalledKeyPackage();
// Send keyPackage to the Discord voice gateway as required by the DAVE flow.

const encrypted = manager.encrypt(MediaType.Audio, ssrc, opusFrame);
const decrypted = manager.decrypt(MediaType.Audio, otherUserId, receivedFrame);

// Both frame methods return Buffer | null.
```

`selfUserId`, `otherUserId`, and `channelId` are Discord snowflakes. The manager constructor accepts the group/channel ID as a decimal string or a JavaScript `bigint`. The binding deliberately accepts strings so 64-bit identifiers are not rounded by JavaScript's `number` representation.

## CommonJS

```js
const {
  DaveSessionManager,
  MediaType,
  maxSupportedProtocolVersion,
} = require("@mineygg/libdave-rs");
```

The package publishes ESM and CommonJS entry points and matching declaration files.

## Choose an API layer

Use `DaveSessionManager` for the voice-gateway-oriented integration. It owns transition and epoch bookkeeping and coordinates media encryptors/decryptors.

Use `Session`, `Encryptor`, and `Decryptor` when you need to manage the MLS session and media cryptography separately. These lower-level classes require the caller to coordinate the protocol flow correctly.
