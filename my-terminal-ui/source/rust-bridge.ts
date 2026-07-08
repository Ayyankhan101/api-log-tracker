import {spawn, type ChildProcess} from 'node:child_process';
import * as fs from 'node:fs';
import * as path from 'node:path';

type ProcessCallbacks = {
	onStdout?: (data: string) => void;
	onStderr?: (data: string) => void;
	onExit?: (code: number | null) => void;
	onError?: (error: Error) => void;
};

function findBinary(): string {
	// Check common locations for the Rust binary
	const candidates = [
		path.resolve(process.cwd(), '../target/debug/api_log_tracker'),
		path.resolve(process.cwd(), 'target/debug/api_log_tracker'),
		path.resolve(process.cwd(), '../../target/debug/api_log_tracker'),
	];

	for (const candidate of candidates) {
		if (fs.existsSync(candidate)) {
			return candidate;
		}
	}

	// Fall back to PATH
	return 'api_log_tracker';
}

export function startServer(callbacks: ProcessCallbacks): ChildProcess {
	const binary = findBinary();
	const proc = spawn(binary, ['serve'], {
		stdio: ['ignore', 'pipe', 'pipe'],
		cwd: path.resolve(process.cwd(), '..'),
	});

	proc.stdout?.on('data', (data: Buffer) => {
		callbacks.onStdout?.(data.toString());
	});

	proc.stderr?.on('data', (data: Buffer) => {
		callbacks.onStderr?.(data.toString());
	});

	proc.on('exit', code => {
		callbacks.onExit?.(code);
	});

	proc.on('error', error => {
		callbacks.onError?.(error);
	});

	return proc;
}

export function runDemoClient(callbacks: ProcessCallbacks): ChildProcess {
	const binary = findBinary();
	const proc = spawn(binary, ['demo-client'], {
		stdio: ['ignore', 'pipe', 'pipe'],
		cwd: path.resolve(process.cwd(), '..'),
	});

	proc.stdout?.on('data', (data: Buffer) => {
		callbacks.onStdout?.(data.toString());
	});

	proc.stderr?.on('data', (data: Buffer) => {
		callbacks.onStderr?.(data.toString());
	});

	proc.on('exit', code => {
		callbacks.onExit?.(code);
	});

	proc.on('error', error => {
		callbacks.onError?.(error);
	});

	return proc;
}

export function runAnalyze(_apiKey: string, provider: string, callbacks: ProcessCallbacks): ChildProcess {
	const binary = findBinary();
	const proc = spawn(binary, ['analyze', '--provider', provider], {
		stdio: ['ignore', 'pipe', 'pipe'],
		cwd: path.resolve(process.cwd(), '..'),
		env: {...process.env, LLM_PROVIDER: provider},
	});

	proc.stdout?.on('data', (data: Buffer) => {
		callbacks.onStdout?.(data.toString());
	});

	proc.stderr?.on('data', (data: Buffer) => {
		callbacks.onStderr?.(data.toString());
	});

	proc.on('exit', code => {
		callbacks.onExit?.(code);
	});

	proc.on('error', error => {
		callbacks.onError?.(error);
	});

	return proc;
}
