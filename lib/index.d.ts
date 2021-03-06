export function decode<T = any>(str: string, max_decompressed_size?: number): Promise<T>;
export function encode(value: any): Promise<string>;
export function decodeSync<T = any>(str: string, max_decompressed_size?: number): T;
export function encodeSync(value: any): string;
