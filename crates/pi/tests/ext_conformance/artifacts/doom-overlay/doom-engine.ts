/**
 * DOOM Engine - WebAssembly wrapper for doomgeneric
 */

import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const DOOM_WAD_PATH = "/doom/doom1.wad";
const WASM_MEMORY_EXPORT = "memory";

type PiWasmHost = typeof globalThis & {
	__pi_wasm_last_instance_id?: number;
	__pi_wasm_stage_file_native?: (path: string, bytes: ArrayLike<number>) => number;
	__pi_wasm_memory_read_native?: (
		instanceId: number,
		memoryName: string,
		offset: number,
		length: number,
	) => ArrayLike<number>;
	__pi_wasm_memory_write_native?: (
		instanceId: number,
		memoryName: string,
		offset: number,
		bytes: ArrayLike<number>,
	) => number;
};

export interface DoomModule {
	_doomgeneric_Create: (argc: number, argv: number) => void;
	_doomgeneric_Tick: () => void;
	_DG_GetFrameBuffer: () => number;
	_DG_GetScreenWidth: () => number;
	_DG_GetScreenHeight: () => number;
	_DG_PushKeyEvent: (pressed: number, key: number) => void;
	_malloc: (size: number) => number;
	_free: (ptr: number) => void;
	__pi_instance_id?: number;
}

export class DoomEngine {
	private module: DoomModule | null = null;
	private frameBufferPtr = 0;
	private initialized = false;
	private instanceId = 0;
	private readonly wadPath: string;
	private _width = 640;
	private _height = 400;
	private readonly progress?: (message: string) => void;

	constructor(wadPath: string, progress?: (message: string) => void) {
		this.wadPath = wadPath;
		this.progress = progress;
	}

	get width(): number {
		return this._width;
	}

	get height(): number {
		return this._height;
	}

	private async yieldAfterProgress(message: string): Promise<void> {
		this.progress?.(message);
		await new Promise((resolve) => setTimeout(resolve, 0));
	}

	private host(): PiWasmHost {
		return globalThis as PiWasmHost;
	}

	private requireStageFile(): NonNullable<PiWasmHost["__pi_wasm_stage_file_native"]> {
		const stageFile = this.host().__pi_wasm_stage_file_native;
		if (typeof stageFile !== "function") {
			throw new Error("PiWasm staging helper is unavailable");
		}
		return stageFile;
	}

	private requireMemoryRead(): NonNullable<PiWasmHost["__pi_wasm_memory_read_native"]> {
		const memoryRead = this.host().__pi_wasm_memory_read_native;
		if (typeof memoryRead !== "function") {
			throw new Error("PiWasm live-memory read helper is unavailable");
		}
		return memoryRead;
	}

	private requireMemoryWrite(): NonNullable<PiWasmHost["__pi_wasm_memory_write_native"]> {
		const memoryWrite = this.host().__pi_wasm_memory_write_native;
		if (typeof memoryWrite !== "function") {
			throw new Error("PiWasm live-memory write helper is unavailable");
		}
		return memoryWrite;
	}

	private writeBytes(offset: number, bytes: ArrayLike<number>): void {
		if (!this.instanceId) {
			throw new Error("DOOM memory write attempted before instance initialization");
		}
		this.requireMemoryWrite()(this.instanceId, WASM_MEMORY_EXPORT, offset, bytes);
	}

	private readBytes(offset: number, length: number): Uint8Array {
		if (!this.instanceId) {
			throw new Error("DOOM memory read attempted before instance initialization");
		}
		return new Uint8Array(this.requireMemoryRead()(this.instanceId, WASM_MEMORY_EXPORT, offset, length));
	}

	private writeCString(ptr: number, text: string): void {
		const bytes = new Array<number>(text.length + 1);
		for (let index = 0; index < text.length; index++) {
			bytes[index] = text.charCodeAt(index) & 0xff;
		}
		bytes[text.length] = 0;
		this.writeBytes(ptr, bytes);
	}

	private writeU32(ptr: number, value: number): void {
		this.writeBytes(ptr, [
			value & 0xff,
			(value >>> 8) & 0xff,
			(value >>> 16) & 0xff,
			(value >>> 24) & 0xff,
		]);
	}

	async init(): Promise<void> {
		await this.yieldAfterProgress("DOOM init: locating WASM build...");

		const __dirname = dirname(fileURLToPath(import.meta.url));
		const buildDir = join(__dirname, "doom", "build");
		const doomJsPath = join(buildDir, "doom.js");

		if (!existsSync(doomJsPath)) {
			throw new Error(`WASM not found at ${doomJsPath}. Run ./doom/build.sh first`);
		}

		await this.yieldAfterProgress("DOOM init: reading WAD file...");
		const wadBytes = new Uint8Array(readFileSync(this.wadPath));

		await this.yieldAfterProgress("DOOM init: staging WAD...");
		this.requireStageFile()(DOOM_WAD_PATH, wadBytes);

		await this.yieldAfterProgress("DOOM init: loading generated JS glue...");
		const doomJsCode = readFileSync(doomJsPath, "utf-8");
		const moduleExports: { exports: unknown } = { exports: {} };
		const nativeRequire = createRequire(doomJsPath);
		const moduleFunc = new Function("module", "exports", "__dirname", "__filename", "require", doomJsCode);
		moduleFunc(moduleExports, moduleExports.exports, buildDir, doomJsPath, nativeRequire);
		const createDoomModule = moduleExports.exports as (config: unknown) => Promise<DoomModule>;

		await this.yieldAfterProgress("DOOM init: instantiating module...");
		this.module = await createDoomModule({
			locateFile: (path: string) => (path.endsWith(".wasm") ? join(buildDir, path) : path),
			print: () => {},
			printErr: () => {},
		});
		if (!this.module) {
			throw new Error("Failed to initialize DOOM module");
		}

		this.instanceId = this.module.__pi_instance_id ?? Number(this.host().__pi_wasm_last_instance_id ?? 0);
		if (!this.instanceId) {
			throw new Error("PiWasm did not expose a live instance id for DOOM");
		}

		await this.yieldAfterProgress("DOOM init: booting engine...");
		this.initDoom();

		this.frameBufferPtr = this.module._DG_GetFrameBuffer();
		this._width = this.module._DG_GetScreenWidth();
		this._height = this.module._DG_GetScreenHeight();
		this.initialized = true;

		await this.yieldAfterProgress(`DOOM init: framebuffer ready (${this._width}x${this._height}).`);
	}

	private initDoom(): void {
		if (!this.module || !this.instanceId) {
			return;
		}

		const args = ["doom", "-iwad", DOOM_WAD_PATH];
		const argPtrs = args.map((arg) => {
			const ptr = this.module!._malloc(arg.length + 1);
			this.writeCString(ptr, arg);
			return ptr;
		});
		const argvPtr = this.module._malloc(argPtrs.length * 4);

		try {
			for (let index = 0; index < argPtrs.length; index++) {
				this.writeU32(argvPtr + index * 4, argPtrs[index]!);
			}
			this.module._doomgeneric_Create(args.length, argvPtr);
		} finally {
			for (const ptr of argPtrs) {
				this.module._free(ptr);
			}
			this.module._free(argvPtr);
		}
	}

	tick(): void {
		if (!this.module || !this.initialized) {
			return;
		}
		this.module._doomgeneric_Tick();
	}

	getFrameRGBA(): Uint8Array {
		if (!this.module || !this.initialized) {
			return new Uint8Array(this._width * this._height * 4);
		}

		const raw = this.readBytes(this.frameBufferPtr, this._width * this._height * 4);
		const rgba = new Uint8Array(raw.length);
		for (let offset = 0; offset < raw.length; offset += 4) {
			rgba[offset + 0] = raw[offset + 2] ?? 0;
			rgba[offset + 1] = raw[offset + 1] ?? 0;
			rgba[offset + 2] = raw[offset + 0] ?? 0;
			rgba[offset + 3] = raw[offset + 3] ?? 255;
		}
		return rgba;
	}

	pushKey(pressed: boolean, key: number): void {
		if (!this.module || !this.initialized) {
			return;
		}
		this.module._DG_PushKeyEvent(pressed ? 1 : 0, key);
	}

	isInitialized(): boolean {
		return this.initialized;
	}
}
