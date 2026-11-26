import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm', 'cjs'],
  dts: true,
  clean: true,
  sourcemap: true,
  target: 'node18',
  // The napi loader resolves the platform binary relative to itself, so it
  // must stay outside the bundle. `src/` and `dist/` sit at the same depth.
  external: ['../binding.cjs'],
});
