/* tslint:disable */
/* eslint-disable */
export function get_registers(): any;
export function get_rodata(): any;
export function get_memory(): any;
export function assemble(assembly: string): Uint8Array;
export function initialize(assembly: string): bigint;
export function set_instruction_data(data: Uint8Array): void;
export function run(assembly: string): bigint;
export function step(): number;
export function get_line_number(): number;
export function is_exited(): boolean;
export function clear_log(): void;
export function get_log(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly get_registers: () => any;
  readonly get_rodata: () => any;
  readonly get_memory: () => any;
  readonly assemble: (a: number, b: number) => [number, number, number, number];
  readonly initialize: (a: number, b: number) => [bigint, number, number];
  readonly set_instruction_data: (a: number, b: number) => void;
  readonly run: (a: number, b: number) => [bigint, number, number];
  readonly step: () => number;
  readonly get_line_number: () => number;
  readonly is_exited: () => number;
  readonly clear_log: () => void;
  readonly get_log: () => [number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
