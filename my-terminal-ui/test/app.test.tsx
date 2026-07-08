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

test('renders footer with key hints', () => {
	const {lastFrame} = render(<App csvPath="test.csv" />);
	const frame = lastFrame();
	expect(frame).toContain('switch');
	expect(frame).toContain('quit');
});

test('renders tab bar', () => {
	const {lastFrame} = render(<App csvPath="test.csv" />);
	const frame = lastFrame();
	expect(frame).toContain('Dashboard');
	expect(frame).toContain('Analysis');
	expect(frame).toContain('Integrate');
});
