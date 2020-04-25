# node-weakauras-parser

## Installation

```bash
npm install node-weakauras-parser
# or
yarn add node-weakauras-parser
```

The package is pre-built for the following environments ([Haswell](https://en.wikipedia.org/wiki/Haswell_(microarchitecture)) is used as the target arch):

|            OS            | Node 10 | Node 12 | Node 13 | Node 14 |
|--------------------------|---------|---------|---------|---------|
|   Linux glibc (x86_64)   |    ✔️    |    ✔️    |    ✔️    |    ✔️    |
| Linux musl-libc (x86_64) |    ✔️    |    ✔️    |    ✔️    |    ✔️    |
|      macOS (x86_64)      |    ✔️    |    ✔️    |    ✔️    |    ✔️    |
|     Windows (x86_64)     |    ✔️    |    ✔️    |    ✔️    |    ✔️    |

If you use something else, you will need [Rust](https://www.rust-lang.org/tools/install) and [zlib](https://www.zlib.net/) in order to build from source code.

If you are getting [SIGILL](https://en.wikipedia.org/wiki/Signal_(IPC)#SIGILL), your CPU does not support some of the instructions that Haswell does. To fix that, you will have to build from source code.

## Usage

```javascript
const parser = require('node-weakauras-parser');

const source = { test: 1 };
const encoded = parser.encode(source);
const decoded = parser.decode(encoded);

console.log(JSON.stringify(source) === JSON.stringify(decoded));
```

Please note that when arrays are involved, encoding them is lossy:

```javascript
const parser = require('node-weakauras-parser');

const source = { test: [true, false] };
const encoded = parser.encode(source);
const decoded = parser.decode(encoded);

// Prints "{ test: { 1: true, 2: false } }"
console.log(decoded);
```

## License

The project is licensed under MIT License, unless stated otherwise in a source file.
