# node-weakauras-parser

## Installation

```bash
npm install node-weakauras-parser
# or
yarn add node-weakauras-parser
```

The package is pre-built for the following environments:

|       OS        | Node 8 | Node 10 | Node 12 | Node 13 |
|-----------------|--------|---------|---------|---------|
|   Linux glibc   |   ✔️    |    ✔️    |    ✔️    |    ✔️    |
| Linux musl-libc |   ✔️    |    ✔️    |    ✔️    |    ✔️    |
|      macOS      |   ✔️    |    ✔️    |    ✔️    |    ✔️    |

If you use something else, you will need [Rust](https://www.rust-lang.org/tools/install) and [zlib](https://www.zlib.net/) in order to build from source code.

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
