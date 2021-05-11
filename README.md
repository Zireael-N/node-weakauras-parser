# node-weakauras-parser

## Installation

```bash
npm install node-weakauras-parser
# or
yarn add node-weakauras-parser
```

The package is pre-built for the following environments ([Haswell](https://en.wikipedia.org/wiki/Haswell_(microarchitecture)) is used as the target arch):

|            OS            | Node 10 | Node 12 | Node 14 | Node 15 | Node 16 |
|--------------------------|---------|---------|---------|---------|---------|
|   Linux glibc (x86_64)   |    ✔️    |    ✔️    |    ✔️    |    ✔️    |    ✔️    |
| Linux musl-libc (x86_64) |    ✔️    |    ✔️    |    ✔️    |    ✔️    |    ✔️    |
|      macOS (x86_64)      |    ✔️    |    ✔️    |    ✔️    |    ✔️    |    ✔️    |
|     Windows (x86_64)     |    ✔️    |    ✔️    |    ✔️    |    ✔️    |    ✔️    |

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

## License

The project is licensed under MIT License, unless stated otherwise in a source file.
