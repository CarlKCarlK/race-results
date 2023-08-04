/* tslint:disable */
/* eslint-disable */
/**
* @param {string} members
* @param {string} race_results
* @param {number} include_city
* @returns {any}
*/
export function member_match(members: string, race_results: string, include_city: number): any;
/**
* @returns {any}
*/
export function sample_members(): any;
/**
* @returns {any}
*/
export function sample_results(): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly member_match: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly sample_members: () => number;
  readonly sample_results: () => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
