# node-weakauras-parser

## Installation

```bash
npm install node-weakauras-parser
# or
yarn add node-weakauras-parser
```

The package is pre-built for the following environments:

|          OS           | Node 8 | Node 10 | Node 12 | Node 13 |
|-----------------------|--------|---------|---------|---------|
|   Linux glibc (x64)   |   ✔️    |    ✔️    |    ✔️    |    ✔️    |
| Linux musl-libc (x64) |   ✔️    |    ✔️    |    ✔️    |    ✔️    |
|      macOS (x64)      |   ✔️    |    ✔️    |    ✔️    |    ✔️    |
|     Windows (x64)     |   ✔️    |    ✔️    |    ✔️    |    ✔️    |

If you use something else, you will need [Rust](https://www.rust-lang.org/tools/install) and [zlib](https://www.zlib.net/) in order to build from source code.

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

Please note that when arrays are involved, encoding them is lossy:

```javascript
const parser = require('node-weakauras-parser');

(async function() {
    const source = { test: [true, false] };
    const encoded = await parser.encode(source);
    const decoded = await parser.decode(encoded);

    // Prints "{ test: { 1: true, 2: false } }"
    console.log(decoded);
}());
```

## Major changes

### v3

- `encode()` and `decode()` in v2 still spent majority of their time on the main thread, thus blocking the event loop. This is no longer the case but **undefined, Infinity and NaN are no longer supported**;
- Functions now return proper Error objects instead of strings.

### v2

- `encode()` and `decode()` are now non-blocking;
- Old, blocking, implementations are available as `encodeSync()` and `decodeSync()`.

## License

The project is licensed under MIT License, unless stated otherwise in a source file.
