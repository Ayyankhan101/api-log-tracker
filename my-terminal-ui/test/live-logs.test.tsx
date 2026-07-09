import {test, expect} from 'vitest';
import {render} from 'ink-testing-library';
import React from 'react';
import LiveLogs from '../source/components/LiveLogs.js';
import type {LogEntry} from '../source/types.js';

function makeEntry(overrides?: Partial<LogEntry>): LogEntry {
	return {
		id: '1',
		timestamp: '2025-01-15T10:30:00Z',
		source: 'server',
		method: 'GET',
		endpoint: '/api/users',
		status_code: 200,
		latency_ms: 150,
		request_size: 0,
		response_size: 1024,
		error: null,
		...overrides,
	};
}

test('renders LIVE LOGS header', () => {
	const {lastFrame} = render(<LiveLogs entries={[]} />);
	expect(lastFrame()).toContain('LIVE LOGS');
});

test('shows entry count', () => {
	const entries = [makeEntry(), makeEntry({id: '2'})];
	const {lastFrame} = render(<LiveLogs entries={entries} />);
	expect(lastFrame()).toContain('2 entries');
});

test('shows filter option', () => {
	const {lastFrame} = render(<LiveLogs entries={[]} />);
	expect(lastFrame()).toContain('filter:');
	expect(lastFrame()).toContain('(f)');
});

test('shows empty state when no entries', () => {
	const {lastFrame} = render(<LiveLogs entries={[]} />);
	expect(lastFrame()).toContain('No data');
});

test('shows entry with method and endpoint', () => {
	const entries = [makeEntry({method: 'POST', endpoint: '/api/data'})];
	const {lastFrame} = render(<LiveLogs entries={entries} />);
	const frame = lastFrame();
	expect(frame).toContain('POST');
	expect(frame).toContain('/api/data');
});

test('shows status code', () => {
	const entries = [makeEntry({status_code: 404})];
	const {lastFrame} = render(<LiveLogs entries={entries} />);
	expect(lastFrame()).toContain('[404]');
});

test('shows latency', () => {
	const entries = [makeEntry({latency_ms: 250})];
	const {lastFrame} = render(<LiveLogs entries={entries} />);
	expect(lastFrame()).toContain('250ms');
});

test('shows column headers', () => {
	const {lastFrame} = render(<LiveLogs entries={[]} />);
	const frame = lastFrame();
	expect(frame).toContain('TIME');
	expect(frame).toContain('SRC');
	expect(frame).toContain('METHOD');
	expect(frame).toContain('ENDPOINT');
	expect(frame).toContain('STS');
	expect(frame).toContain('LATENCY');
});

test('shows source column', () => {
	const entries = [makeEntry({source: 'client'})];
	const {lastFrame} = render(<LiveLogs entries={entries} />);
	expect(lastFrame()).toContain('client');
});
