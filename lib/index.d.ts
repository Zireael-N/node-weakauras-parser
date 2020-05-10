export function decode<T = any>(str: string, max_decompressed_size?: number): Promise<T>;
export function encode(value: any): Promise<string>;
