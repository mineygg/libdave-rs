# `Session`

Lower-level DAVE MLS session wrapper. Use this when the application needs to drive MLS processing directly rather than using `DaveSessionManager`.

## Constructor

```ts
new Session(authSessionId?: string)
```

Creates a session. `authSessionId` is optional.

## Initialization and protocol

### `init(version: number, groupId: string | bigint, selfUserId: string): void`

Initializes the session for a voice-channel/group ID and local user ID.

### `reset(): void`

Resets the session state.

### `setProtocolVersion(version: number): void`

Sets the protocol version.

### `protocolVersion: number` (getter)

Returns the current protocol version.

### `setExternalSender(externalSender: Buffer): void`

Sets the MLS external-sender data.

## Recognized users

### `addRecognizedUser(userId: string): void`

Marks a user as connected/recognized. MLS messages naming users outside the recognized set are rejected.

### `removeRecognizedUser(userId: string): void`

Removes a user from the recognized set.

## MLS processing

### `processProposals(proposals: Buffer): Buffer | null`

Processes proposals and returns commit/welcome response bytes when produced, or `null` when no response bytes are produced.

### `processCommit(commit: Buffer): CommitOutcome`

Processes an MLS commit.

```ts
interface CommitOutcome {
  failed: boolean;
  ignored: boolean;
  roster: string[]; // user IDs added, changed or removed by the commit
  rosterSignatures: Record<string, Buffer>; // signature key per ID; empty if removed
}
```

### `processWelcome(welcome: Buffer): WelcomeOutcome | null`

Processes a Welcome message. Returns `null` if it could not be processed.

```ts
interface WelcomeOutcome {
  roster: string[]; // user IDs in the joined group
  rosterSignatures: Record<string, Buffer>; // signature key per ID
}
```

## Key material and identity

### `getMarshalledKeyPackage(): Buffer`

Returns the marshalled key package.

### `getKeyRatchet(userId: string): KeyRatchet | null`

Returns the media key ratchet for a user when available. The underlying Rust operation may report an error, which is surfaced as a JavaScript exception.

### `getLastEpochAuthenticator(): Buffer`

Returns the last epoch authenticator.

### `getPairwiseFingerprint(version: number, userId: string): Promise<Buffer>`

Asynchronously derives the identity fingerprint shared with `userId`. The native task computes this away from the JavaScript main thread. It rejects if there is no MLS group or the user is unknown.

## MLS failures

### `takeFailures(): MlsFailure[]`

Returns and clears queued MLS failures.

### `destroy(): void`

Immediately releases the native session. Subsequent operations requiring the native session throw a destroyed-object error.

## Choosing this class

`Session` exposes MLS operations and key-derivation helpers. It does not itself provide the manager's voice-gateway transition/epoch bookkeeping or the convenience per-user media routing of `DaveSessionManager`. Applications using it directly must coordinate those responsibilities.
