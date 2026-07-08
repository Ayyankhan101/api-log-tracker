import * as fs from 'node:fs';
import chokidar from 'chokidar';
import type {LogEntry} from './types.js';

type WatcherCallbacks = {
	onEntry: (entry: LogEntry) => void;
	onBatch: (entries: LogEntry[]) => void;
};

function parseLine(line: string): LogEntry | null {
	const parts: string[] = [];
	let current = '';
	let inQuotes = false;

	for (const char of line) {
		if (char === '"') {
			inQuotes = !inQuotes;
		} else if (char === ',' && !inQuotes) {
			parts.push(current);
			current = '';
		} else {
			current += char;
		}
	}

	parts.push(current);

	if (parts.length < 10) {
		return null;
	}

	const id = parts[0] ?? '';
	const timestamp = parts[1] ?? '';
	const source = parts[2] ?? '';
	const method = parts[3] ?? '';
	const endpoint = parts[4] ?? '';
	const statusCode = parts[5] ?? '0';
	const latencyMs = parts[6] ?? '0';
	const requestSize = parts[7] ?? '0';
	const responseSize = parts[8] ?? '0';
	const error = parts.slice(9).join(',') || null;

	return {
		id,
		timestamp,
		source: source as 'server' | 'client',
		method,
		endpoint,
		status_code: Number.parseInt(statusCode, 10) || 0,
		latency_ms: Number.parseInt(latencyMs, 10) || 0,
		request_size: Number.parseInt(requestSize, 10) || 0,
		response_size: Number.parseInt(responseSize, 10) || 0,
		error,
	};
}

function loadExistingEntries(csvPath: string): LogEntry[] {
	if (!fs.existsSync(csvPath)) {
		return [];
	}

	const entries: LogEntry[] = [];
	const content = fs.readFileSync(csvPath, 'utf-8');
	const lines = content.split('\n').slice(1); // skip header

	for (const line of lines) {
		const trimmed = line.trim();
		if (trimmed) {
			const entry = parseLine(trimmed);
			if (entry) {
				entries.push(entry);
			}
		}
	}

	return entries;
}

export function createCsvWatcher(csvPath: string, callbacks: WatcherCallbacks) {
	// Load existing entries first
	const existing = loadExistingEntries(csvPath);
	if (existing.length > 0) {
		callbacks.onBatch(existing);
	}

	let lastSize = fs.existsSync(csvPath) ? fs.statSync(csvPath).size : 0;

	const watcher = chokidar.watch(csvPath, {
		persistent: true,
		ignoreInitial: true,
		awaitWriteFinish: {stabilityThreshold: 100, pollInterval: 50},
	});

	watcher.on('change', () => {
		try {
			if (!fs.existsSync(csvPath)) {
				return;
			}

			const stat = fs.statSync(csvPath);
			if (stat.size <= lastSize) {
				lastSize = stat.size;
				return;
			}

			// Read only the new bytes
			const fd = fs.openSync(csvPath, 'r');
			const buffer = Buffer.alloc(stat.size - lastSize);
			fs.readSync(fd, buffer, 0, buffer.length, lastSize);
			fs.closeSync(fd);

			lastSize = stat.size;

			const newContent = buffer.toString('utf-8');
			const lines = newContent.split('\n');

			for (const line of lines) {
				const trimmed = line.trim();
				if (trimmed && !trimmed.startsWith('id,')) {
					const entry = parseLine(trimmed);
					if (entry) {
						callbacks.onEntry(entry);
					}
				}
			}
		} catch {
			// File might be mid-write, ignore
		}
	});

	return {
		close: () => {
			watcher.close();
		},
	};
}

export {parseLine, loadExistingEntries};
