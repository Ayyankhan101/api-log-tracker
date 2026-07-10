import {test, expect} from 'vitest';
import {render} from 'ink-testing-library';
import Integration from '../source/components/Integration.js';

test('renders language tabs', () => {
	const {lastFrame} = render(<Integration />);
	const frame = lastFrame();
	expect(frame).toContain('Python');
	expect(frame).toContain('C');
	expect(frame).toContain('C++');
	expect(frame).toContain('Rust');
	expect(frame).toContain('Go');
	expect(frame).toContain('Erlang');
});

test('renders LLM Providers tab', () => {
	const {lastFrame} = render(<Integration />);
	expect(lastFrame()).toContain('LLM Providers');
});

test('renders integration header', () => {
	const {lastFrame} = render(<Integration />);
	expect(lastFrame()).toContain('INTEGRATION SNIPPETS');
});

test('renders daemon info', () => {
	const {lastFrame} = render(<Integration />);
	const frame = lastFrame();
	expect(frame).toContain('/api/log');
	expect(frame).toContain('/api/analyze');
	expect(frame).toContain('/api/health');
});

test('renders navigation hint', () => {
	const {lastFrame} = render(<Integration />);
	expect(lastFrame()).toContain('j/k');
});

test('renders LLM Providers tab label', () => {
	const {lastFrame} = render(<Integration />);
	const frame = lastFrame();
	expect(frame).toContain('LLM Providers');
});

test('renders daemon start hint', () => {
	const {lastFrame} = render(<Integration />);
	expect(lastFrame()).toContain('Daemon');
});

test('renders endpoint list', () => {
	const {lastFrame} = render(<Integration />);
	const frame = lastFrame();
	expect(frame).toContain('/api/log');
	expect(frame).toContain('/api/analyze');
	expect(frame).toContain('/api/health');
	expect(frame).toContain('/api/logs');
});

test('renders CSV path reference', () => {
	const {lastFrame} = render(<Integration />);
	expect(lastFrame()).toContain('api_logs.csv');
});

test('renders code snippet content', () => {
	const {lastFrame} = render(<Integration />);
	const frame = lastFrame();
	expect(frame).toContain('requests.post');
	expect(frame).toContain('localhost:8080');
});
