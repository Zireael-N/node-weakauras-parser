export const enum FormatVersion {
    /** WA-strings that are prefixed with '!'. MDT-compatible. */
    Deflate = 1,
    /** WA-strings that are prefixed with '!WA:2!'. */
    BinarySerialization = 2,
}

export function decode<T = any>(str: string, max_decompressed_size?: number): Promise<T>;
export function encode(value: any, format_version?: FormatVersion): Promise<string>;
export function decodeSync<T = any>(str: string, max_decompressed_size?: number): T;
export function encodeSync(value: any, format_version?: FormatVersion): string;
