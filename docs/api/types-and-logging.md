# Types, statistics, failures, and logging

## `MediaType`

```ts
enum MediaType {
  Audio = 0,
  Video = 1,
}
```

Selects the media type of a frame.

## `Codec`

```ts
enum Codec {
  Unknown = 0,
  Opus = 1,
  Vp8 = 2,
  Vp9 = 3,
  H264 = 4,
  H265 = 5,
  Av1 = 6,
}
```

Codec associated with an SSRC.

## Statistics

All statistic fields are exposed as JavaScript numbers (`f64` in the napi object conversion).

### `EncryptStats`

```ts
interface EncryptStats {
  success: number;
  failure: number;
  passthrough: number;
  missing_key: number;
  attempts: number;
  max_attempts: number;
  duration: number;
}
```

### `DecryptStats`

```ts
interface DecryptStats {
  success: number;
  failure: number;
  passthrough: number;
  missing_key: number;
  invalid_nonce: number;
  attempts: number;
  duration: number;
}
```

The fields mirror counters reported by libdave. The binding does not define units for `duration` in its public wrapper source; consumers should not assume a unit without checking the upstream implementation/version.

## MLS result and failure objects

```ts
interface MlsCommitResult {
  ready: boolean;
  failed: boolean;
}

interface MlsWelcomeResult {
  ready: boolean;
}

interface MlsFailure {
  source: string;
  reason: string;
}

interface CommitOutcome {
  failed: boolean;
  ignored: boolean;
  roster: string[];
}

interface WelcomeOutcome {
  roster: string[];
}
```

`takeFailures()` on `Session` and `DaveSessionManager` drains the queued failure list. The queue retains at most 64 recent entries, dropping the oldest entry when full.

## Logging

### `LogSeverity`

```ts
enum LogSeverity {
  Verbose = 0,
  Info = 1,
  Warning = 2,
  Error = 3,
}
```

### `LogEntry`

```ts
interface LogEntry {
  severity: LogSeverity;
  file: string;
  line: number;
  message: string;
}
```

### `setLogSink(callback?: ((entry: LogEntry) => void) | null): void`

Installs a process-wide callback for libdave log messages. Pass `null` or omit the callback to remove the sink.

Messages are delivered asynchronously. The callback does not keep the Node.js event loop alive.

```ts
import { setLogSink, LogSeverity } from '@mineygg/libdave-rs';

setLogSink((entry) => {
  if (entry.severity === LogSeverity.Error) {
    console.error(`[libdave] ${entry.file}:${entry.line}`, entry.message);
  }
});

// Later:
setLogSink(null);
```

Only one process-wide sink is installed at a time. Installing another replaces the previous sink.
