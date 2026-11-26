# `DaveSessionManager`

Voice-gateway-level DAVE coordinator. It manages protocol transitions, epochs, MLS processing, and media encryptor/decryptor state.

## Constructor

```ts
new DaveSessionManager(selfUserId: string, groupId: string | bigint)
```

- `selfUserId`: local Discord user ID as a decimal string.
- `groupId`: voice-channel/group snowflake as a decimal string or `bigint`.

## Properties

### `protocolVersion: number` (getter)

Current protocol version.

## Membership

### `addUser(userId: string): void`

Adds a user to the manager's known membership.

### `removeUser(userId: string): void`

Removes a user from the manager's known membership.

## Voice-gateway transition callbacks

### `onSelectProtocolAck(protocolVersion: number): void`

Notifies the manager that the voice gateway acknowledged the selected DAVE protocol version.

### `onPrepareTransition(transitionId: number, protocolVersion: number): number | null`

Processes a prepare-transition event. Returns the transition ID to acknowledge, or `null` for transition `0`.

### `onExecuteTransition(transitionId: number): void`

Processes an execute-transition event.

### `onPrepareEpoch(epoch: number | bigint, protocolVersion: number): void`

Prepares the specified epoch and protocol version. The epoch accepts a non-negative integer representable as `u32` or a `bigint` that fits the binding's unsigned 64-bit conversion.

### `onExternalSenderPackage(data: Buffer): void`

Provides the external-sender package bytes to the MLS session.

## MLS message processing

### `onMlsProposals(proposals: Buffer): Buffer | null`

Processes MLS proposals. Returns response bytes when produced, otherwise `null`.

### `onMlsCommit(transitionId: number, commit: Buffer): MlsCommitResult`

Processes an MLS commit for a transition.

`MlsCommitResult`:

```ts
interface MlsCommitResult {
  ready: boolean;
  failed: boolean;
}
```

### `onMlsWelcome(transitionId: number, welcome: Buffer): MlsWelcomeResult`

Processes an MLS Welcome message.

`MlsWelcomeResult`:

```ts
interface MlsWelcomeResult {
  ready: boolean;
}
```

## Key package

### `getMarshalledKeyPackage(): Buffer`

Returns the marshalled MLS key package. The application is responsible for transporting it through the appropriate voice-gateway flow.

## Media

### `encrypt(mediaType: MediaType, ssrc: number, frame: Buffer): Buffer | null`

Encrypts an outgoing media frame for the given SSRC. Returns `null` when the frame cannot be encrypted. The binding converts lower-level encryption errors into `null` for this convenience method.

### `decrypt(mediaType: MediaType, userId: string, frame: Buffer): Buffer | null`

Decrypts a frame attributed to `userId`. Returns `null` when it cannot be decrypted.

### `assignSsrcCodec(ssrc: number, codec: Codec): void`

Associates an SSRC with its codec. This informs the media encryptor about the codec carried by that SSRC.

## Statistics

### `getEncryptStats(mediaType: MediaType): EncryptStats`

Returns aggregate encryption counters for the selected media type.

### `getDecryptStats(userId: string, mediaType: MediaType): DecryptStats | null`

Returns counters for the specified remote user and media type, or `null` if no decryptor state has been set up for that user.

## MLS failures

### `takeFailures(): MlsFailure[]`

Returns and clears failures queued since the previous call. The internal queue is bounded to the most recent 64 entries.

## Lifecycle

### `destroy(): void`

Immediately releases the native manager. Subsequent methods that require the native instance throw a destroyed-object error. Prefer calling this when the manager is no longer needed instead of relying solely on garbage collection.

## Typical flow (schematic)

```ts
const dave = new DaveSessionManager(selfUserId, channelId);

dave.addUser(remoteUserId);
dave.onSelectProtocolAck(maxSupportedProtocolVersion());

const keyPackage = dave.getMarshalledKeyPackage();
// Exchange keyPackage and subsequent MLS/transition messages with the gateway
// according to Discord's DAVE voice protocol.

const output = dave.encrypt(MediaType.Audio, ssrc, encodedFrame);
const input = dave.decrypt(MediaType.Audio, remoteUserId, encryptedFrame);

const failures = dave.takeFailures();
```

This is an API illustration, not a complete Discord voice-gateway implementation. The application must feed all gateway events and MLS messages in the order required by the protocol.
