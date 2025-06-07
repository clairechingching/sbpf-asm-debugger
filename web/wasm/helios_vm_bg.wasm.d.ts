/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const get_registers: () => any;
export const get_rodata: () => any;
export const get_memory: () => any;
export const assemble: (a: number, b: number) => [number, number, number, number];
export const initialize: (a: number, b: number) => [bigint, number, number];
export const set_instruction_data: (a: number, b: number) => void;
export const run: (a: number, b: number) => [bigint, number, number];
export const step: () => number;
export const get_line_number: () => number;
export const is_exited: () => number;
export const clear_log: () => void;
export const get_log: () => [number, number];
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_export_2: WebAssembly.Table;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_start: () => void;
