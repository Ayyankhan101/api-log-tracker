import {test, expect} from 'vitest';
import {render} from 'ink-testing-library';
import App from '../source/app.js';

test('renders header with app name', () => {
	const {lastFrame} = render(<App csvPath="test.csv" />);
	expect(lastFrame()).toContain('api-log-tracker');
});

test('renders version string', () => {
	const {lastFrame} = render(<App csvPath="test.csv" />);
	expect(lastFrame()).toContain('v0.1');
});

test('shows zero entries initially', () => {
	const {lastFrame} = render(<App csvPath="test.csv" />);
	expect(lastFrame()).toContain('0 entries');
});

test('shows zero entries initially', () => {
	const {lastFrame} = render(<App csvPath="test.csv" />);
	expect(lastFrame()).toContain('0 entries');
});

test('shows OFFLINE status', () => {
	const {lastFrame} = render(<App csvPath="test.csv" />);
	expect(lastFrame()).toContain('OFFLINE');
});

test('renders all tab labels', () => {
	const {lastFrame} = render(<App csvPath="test.csv" />);
	const frame = lastFrame();
	expect(frame).toContain('Dashboard');
	expect(frame).toContain('Live Logs');
	expect(frame).toContain('Analysis');
	expect(frame).toContain('Controls');
	expect(frame).toContain('Integrate');
});

test('renders footer with keyboard shortcuts', () => {
	const {lastFrame} = render(<App csvPath="test.csv" />);
	const frame = lastFrame();
	expect(frame).toContain('1-5');
	expect(frame).toContain('switch');
	expect(frame).toContain('q');
	expect(frame).toContain('quit');
});
