import {test, expect} from 'vitest';
import {render} from 'ink-testing-library';
import Dashboard from '../source/components/Dashboard.js';
import type {DashboardStats} from '../source/types.js';

function makeStats(overrides?: Partial<DashboardStats>): DashboardStats {
	return {
		total: 42,
		errors: 3,
		errorRate: 7.1,
		avgLatency: 200,
		maxLatency: 5000,
		statusCodes: {200: 38, 404: 1, 500: 3},
		endpoints: {'/api/users': 25, '/api/data': 17},
		requestsPerMinute: Array(20).fill(2),
		...overrides,
	};
}

test('renders total entries', () => {
	const {lastFrame} = render(<Dashboard stats={makeStats()} />);
	expect(lastFrame()).toContain('42');
});

test('renders error rate', () => {
	const {lastFrame} = render(<Dashboard stats={makeStats()} />);
	expect(lastFrame()).toContain('7.1');
});

test('renders avg latency', () => {
	const {lastFrame} = render(<Dashboard stats={makeStats()} />);
	expect(lastFrame()).toContain('200');
});

test('renders max latency', () => {
	const {lastFrame} = render(<Dashboard stats={makeStats()} />);
	expect(lastFrame()).toContain('5,000');
});

test('renders status code section', () => {
	const {lastFrame} = render(<Dashboard stats={makeStats()} />);
	expect(lastFrame()).toContain('STATUS DISTRIBUTION');
});

test('renders top endpoints section', () => {
	const {lastFrame} = render(<Dashboard stats={makeStats()} />);
	expect(lastFrame()).toContain('TOP ENDPOINTS');
});

test('renders sparkline section', () => {
	const {lastFrame} = render(<Dashboard stats={makeStats()} />);
	expect(lastFrame()).toContain('REQUESTS/MIN');
});

test('renders zero state', () => {
	const {lastFrame} = render(<Dashboard stats={makeStats({total: 0, errors: 0, errorRate: 0, avgLatency: 0, maxLatency: 0, statusCodes: {}, endpoints: {}, requestsPerMinute: []})} />);
	expect(lastFrame()).toContain('0');
});
