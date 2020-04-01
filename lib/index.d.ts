export function decode<T = any>(str: string): Promise<T>;
export function encode(value: any): Promise<string>;
export function decodeSync<T = any>(str: string): T;
export function encodeSync(value: any): string;
