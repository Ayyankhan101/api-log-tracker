import {test, expect} from 'vitest';
import {render} from 'ink-testing-library';
import React from 'react';
import Analysis from '../source/components/Analysis.js';
import type {AnalysisResult} from '../source/types.js';

const idleResult: AnalysisResult = {
	status: 'idle',
	output: '',
	error: null,
	provider: undefined,
};

test('renders ANALYSIS header', () => {
	const {lastFrame} = render(<Analysis result={idleResult} onResult={() => {}} />);
	expect(lastFrame()).toContain('ANALYSIS');
});

test('renders provider selector', () => {
	const {lastFrame} = render(<Analysis result={idleResult} onResult={() => {}} />);
	const frame = lastFrame();
	expect(frame).toContain('OpenAI');
	expect(frame).toContain('Claude');
	expect(frame).toContain('Gemini');
});

test('renders navigation hint', () => {
	const {lastFrame} = render(<Analysis result={idleResult} onResult={() => {}} />);
	expect(lastFrame()).toContain('j/k to switch provider');
});

test('shows idle state with run hint', () => {
	const {lastFrame} = render(<Analysis result={idleResult} onResult={() => {}} />);
	expect(lastFrame()).toContain('press [r] to run');
});

test('shows done state with output', () => {
	const doneResult: AnalysisResult = {
		status: 'done',
		output: 'Analysis looks good. No anomalies detected.',
		error: null,
		provider: 'anthropic',
	};
	const {lastFrame} = render(<Analysis result={doneResult} onResult={() => {}} />);
	const frame = lastFrame();
	expect(frame).toContain('analysis complete');
	expect(frame).toContain('Analysis looks good');
});

test('shows error state', () => {
	const errorResult: AnalysisResult = {
		status: 'error',
		output: 'Connection failed',
		error: 'Exit code 1',
		provider: 'openai',
	};
	const {lastFrame} = render(<Analysis result={errorResult} onResult={() => {}} />);
	const frame = lastFrame();
	expect(frame).toContain('ERROR');
	expect(frame).toContain('Exit code 1');
});

test('shows running result with provider', () => {
	const runningResult: AnalysisResult = {
		status: 'running',
		output: '',
		error: null,
		provider: 'gemini',
	};
	const {lastFrame} = render(<Analysis result={runningResult} onResult={() => {}} />);
	const frame = lastFrame();
	expect(frame).toContain('ANALYSIS');
	expect(frame).toContain('Gemini');
});
