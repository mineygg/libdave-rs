# Media cryptography

## `KeyRatchet`

A per-user media key ratchet obtained from `Session.getKeyRatchet(userId)`.

The current Node.js binding exposes the ratchet as an opaque object. It is passed to an `Encryptor` or `Decryptor`; it has no separately exposed public methods in the binding source.

## `Encryptor`

Encrypts outgoing media frames.

### Constructor

```ts
new Encryptor();
```

### `setKeyRatchet(ratchet: KeyRatchet): void`

Sets the media key ratchet used for encryption.

### `setPassthroughMode(enabled: boolean): void`

Enables or disables passthrough mode.

### `passthroughMode: boolean` (getter)

Reports whether passthrough mode is enabled.

### `hasKeyRatchet: boolean` (getter)

Reports whether a key ratchet has been installed.

### `protocolVersion: number` (getter)

Returns the encryptor's protocol version.

### `setProtocolVersionChangedCallback(callback?: (() => void) | null): void`

Calls `callback` whenever `protocolVersion` changes; pass `null` to remove it. The callback takes no arguments, is delivered asynchronously and does not keep the event loop alive.

### `getMaxCiphertextByteSize(mediaType: MediaType, frameLength: number): number`

Returns the largest output `encrypt` can produce for a frame of that length.

### `assignSsrcToCodec(ssrc: number, codec: Codec): void`

Associates an SSRC with its codec.

### `encrypt(mediaType: MediaType, ssrc: number, frame: Buffer): Buffer`

Encrypts one frame. Unlike `DaveSessionManager.encrypt`, this lower-level method surfaces a native encryption failure as a JavaScript exception rather than returning `null`.

### `getStats(mediaType: MediaType): EncryptStats`

Returns encryption counters for the media type.

### `destroy(): void`

Immediately releases the native encryptor.

## `Decryptor`

Decrypts media frames from one remote sender.

### Constructor

```ts
new Decryptor();
```

### `transitionToKeyRatchet(ratchet: KeyRatchet): void`

Transitions the decryptor to the supplied key ratchet.

### `transitionToPassthroughMode(enabled: boolean): void`

Transitions into or out of passthrough mode.

### `decrypt(mediaType: MediaType, frame: Buffer): Buffer`

Decrypts one frame. Native decryption failures are surfaced as JavaScript exceptions.

### `getMaxPlaintextByteSize(mediaType: MediaType, frameLength: number): number`

Returns the largest output `decrypt` can produce for a frame of that length.

### `getStats(mediaType: MediaType): DecryptStats`

Returns decryption counters for the media type.

### `destroy(): void`

Immediately releases the native decryptor.

## Important distinction

The manager's `encrypt` and `decrypt` convenience methods return `Buffer | null` for an unprocessable frame. The standalone `Encryptor.encrypt` and `Decryptor.decrypt` methods return a `Buffer` on success and throw on native errors.
