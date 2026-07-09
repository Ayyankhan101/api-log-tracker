import {test, expect} from 'vitest';
import {render} from 'ink-testing-library';
import React from 'react';
import ServerControls from '../source/components/ServerControls.js';
import type {ServerStatus} from '../source/types.js';

function renderWithStatus(status: ServerStatus) {
	return render(
		<ServerControls serverStatus={status} onStatusChange={() => {}} />,
	);
}

test('renders SERVER CONTROLS header', () => {
	const {lastFrame} = renderWithStatus('stopped');
	expect(lastFrame()).toContain('SERVER CONTROLS');
});

test('shows stopped status', () => {
	const {lastFrame} = renderWithStatus('stopped');
	expect(lastFrame()).toContain('STOPPED');
});

test('shows running status', () => {
	const {lastFrame} = renderWithStatus('running');
	expect(lastFrame()).toContain('RUNNING');
});

test('shows starting status', () => {
	const {lastFrame} = renderWithStatus('starting');
	expect(lastFrame()).toContain('STARTING');
});

test('shows error status', () => {
	const {lastFrame} = renderWithStatus('error');
	expect(lastFrame()).toContain('ERROR');
});

test('shows start option when stopped', () => {
	const {lastFrame} = renderWithStatus('stopped');
	expect(lastFrame()).toContain('[s] start server');
});

test('shows stop option when running', () => {
	const {lastFrame} = renderWithStatus('running');
	expect(lastFrame()).toContain('[k] stop server');
});

test('shows demo option when running', () => {
	const {lastFrame} = renderWithStatus('running');
	expect(lastFrame()).toContain('[d] run demo client');
});

test('shows clear option always', () => {
	const {lastFrame} = renderWithStatus('stopped');
	expect(lastFrame()).toContain('[c] clear output');
});
