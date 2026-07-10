import {test, expect} from 'vitest';
import {render} from 'ink-testing-library';
import Tabs from '../source/components/Tabs.js';

test('renders all tab labels', () => {
	const {lastFrame} = render(<Tabs active="dashboard" />);
	const frame = lastFrame();
	expect(frame).toContain('Dashboard');
	expect(frame).toContain('Live Logs');
	expect(frame).toContain('Analysis');
	expect(frame).toContain('Controls');
	expect(frame).toContain('Integrate');
});

test('shows key numbers for each tab', () => {
	const {lastFrame} = render(<Tabs active="dashboard" />);
	const frame = lastFrame();
	expect(frame).toContain('1:');
	expect(frame).toContain('2:');
	expect(frame).toContain('3:');
	expect(frame).toContain('4:');
	expect(frame).toContain('5:');
});

test('highlights active tab', () => {
	const {lastFrame} = render(<Tabs active="analysis" />);
	const frame = lastFrame();
	expect(frame).toContain('3:Analysis');
});

test('highlights different active tab', () => {
	const {lastFrame} = render(<Tabs active="integration" />);
	const frame = lastFrame();
	expect(frame).toContain('5:Integrate');
});

test('highlights each tab when active', () => {
	const dash = render(<Tabs active="dashboard" />);
	expect(dash.lastFrame()).toContain('1:Dashboard');
	const logs = render(<Tabs active="logs" />);
	expect(logs.lastFrame()).toContain('2:Live Logs');
	const analysis = render(<Tabs active="analysis" />);
	expect(analysis.lastFrame()).toContain('3:Analysis');
	const ctrl = render(<Tabs active="controls" />);
	expect(ctrl.lastFrame()).toContain('4:Controls');
	const integ = render(<Tabs active="integration" />);
	expect(integ.lastFrame()).toContain('5:Integrate');
});
