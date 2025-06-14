# node-weakauras-parser

## Installation

```bash
npm install node-weakauras-parser
# or
yarn add node-weakauras-parser
```

The package is pre-built for the following environments ([Haswell](https://en.wikipedia.org/wiki/Haswell_(microarchitecture)) is used as the target arch):

|            OS            | Node >=12 |
|--------------------------|-----------|
|   Linux glibc (x86_64)   |     ✔️     |
| Linux musl-libc (x86_64) |     ✔️     |
|      macOS (x86_64)      |     ✔️     |
|     Windows (x86_64)     |     ✔️     |

If you use something else, you will need [Rust](https://www.rust-lang.org/tools/install) and [zlib](https://www.zlib.net/) in order to build from source code.

If you are getting [SIGILL](https://en.wikipedia.org/wiki/Signal_(IPC)#SIGILL), your CPU does not support some of the instructions that Haswell does. To fix that, you will have to build from source code.

## Usage

Non-blocking version:

```javascript
const parser = require('node-weakauras-parser');

(async function() {
    const source = { test: 1 };
    const encoded = await parser.encode(source);
    const decoded = await parser.decode(encoded);

    console.log(JSON.stringify(source) === JSON.stringify(decoded));
}());
```

Blocking version (slightly faster, but [blocks the event loop](https://nodejs.org/en/docs/guides/dont-block-the-event-loop/)):

```javascript
const parser = require('node-weakauras-parser');

const source = { test: 1 };
const encoded = parser.encodeSync(source);
const decoded = parser.decodeSync(encoded);

console.log(JSON.stringify(source) === JSON.stringify(decoded));
```

Both `decode()` and `decodeSync()` accept an optional argument to configure the memory usage limit for decompression (in bytes). Default value is 8 MBs. You can pass `+Infinity` to disable it.

Both `encode` and `encodeSync()` accept an optional argument to specify the encoding version. See the definition of FormatVersion in [index.d.ts](https://github.com/Zireael-N/node-weakauras-parser/blob/master/lib/index.d.ts).

## Known issues

- Table references from LibSerialize are not fully supported.
  For example, self-referential tables (or tables referencing an ancestor) will cause an error.
  As far as I know, those cannot be stored in `SavedVariables.lua`, so it shouldn't be an issue with WA strings.

## Major changes

### v3.2

- This package now uses [Node-API v4](https://nodejs.org/api/n-api.html#node-api-version-matrix) (Node.js 10.16.0, 11.8.0, 12.0.0 and later) instead of [Native Abstractions for Node.js](https://github.com/nodejs/nan) to ensure ABI compatibility with future versions of Node.js.

### v3.1

- `encode()` now uses a new serialization algorithm adopted by WA in v2.18.

### v3

- `encode()` and `decode()` in v2 still spent majority of their time on the main thread, thus blocking the event loop. This is no longer the case but **infinite numbers are no longer supported**;
- Functions now return proper Error objects instead of strings.

### v2

- `encode()` and `decode()` are now non-blocking;
- Old, blocking, implementations are available as `encodeSync()` and `decodeSync()`.

## License

The source code is licensed under the MIT License. However, it depends on GPL-licensed code, so the whole distribution is licensed under GPLv2.
