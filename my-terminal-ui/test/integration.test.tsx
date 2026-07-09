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
