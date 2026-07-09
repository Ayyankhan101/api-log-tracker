import React, {useRef, useState} from 'react';
import {Box, Text, useInput} from 'ink';
import type {AnalysisResult} from '../types.js';
import {runAnalyze} from '../rust-bridge.js';

const PROVIDERS = [
	{id: 'openai', label: 'OpenAI', key: 'OPENAI_API_KEY'},
	{id: 'anthropic', label: 'Claude', key: 'ANTHROPIC_API_KEY'},
	{id: 'gemini', label: 'Gemini', key: 'GEMINI_API_KEY'},
	{id: 'grok', label: 'Grok (xAI)', key: 'XAI_API_KEY'},
	{id: 'groq', label: 'Groq', key: 'GROQ_API_KEY'},
	{id: 'deepseek', label: 'DeepSeek', key: 'DEEPSEEK_API_KEY'},
	{id: 'qwen', label: 'Qwen', key: 'DASHSCOPE_API_KEY'},
	{id: 'baichuan', label: 'Baichuan', key: 'BAICHUAN_API_KEY'},
	{id: 'yi', label: 'Yi', key: 'YI_API_KEY'},
	{id: 'stepfun', label: 'StepFun', key: 'STEPFUN_API_KEY'},
	{id: 'glm', label: 'GLM (Zhipu)', key: 'ZHIPU_API_KEY'},
	{id: 'ernie', label: 'ERNIE (Baidu)', key: 'BAIDU_API_KEY'},
];

type Props = {
	result: AnalysisResult;
	onResult: (result: AnalysisResult) => void;
};

export default function Analysis({result, onResult}: Props) {
	const [loading, setLoading] = useState(false);
	const [providerIdx, setProviderIdx] = useState(0);
	const outputRef = useRef('');

	useInput(input => {
		if (input === 'j' || input === 'ArrowDown') {
			setProviderIdx(i => (i + 1) % PROVIDERS.length);
		}

		if (input === 'k' || input === 'ArrowUp') {
			setProviderIdx(i => (i - 1 + PROVIDERS.length) % PROVIDERS.length);
		}

		if (input === 'r' && !loading) {
			setLoading(true);
			outputRef.current = '';
			const provider = PROVIDERS[providerIdx]!;
			const apiKey = process.env[provider.key] || '';
			onResult({status: 'running', output: '', error: null, provider: provider.id});

			runAnalyze(apiKey, provider.id, {
				onStdout(data) {
					outputRef.current += data;
				},
				onStderr(data) {
					outputRef.current += data;
				},
				onExit(code) {
					setLoading(false);
					const output = outputRef.current;
					if (code === 0) {
						onResult({status: 'done', output, error: null, provider: provider.id});
					} else {
						onResult({status: 'error', output, error: `Exit code ${code}`, provider: provider.id});
					}
				},
				onError(error) {
					setLoading(false);
					onResult({status: 'error', output: '', error: error.message, provider: provider.id});
				},
			});
		}
	});

	const provider = PROVIDERS[providerIdx]!;
	const apiKey = process.env[provider.key] || '';

	return (
		<Box flexDirection="column" gap={1}>
			<Box gap={0}>
				<Text color="green">{'╔═ ANALYSIS '}</Text>
				<Text color="gray">{'API key: '}</Text>
				{apiKey ? (
					<Text color="green">{'[SET] '}</Text>
				) : (
					<Text color="red">{'[NOT SET] '}</Text>
				)}
				<Text color="green">{'═'.repeat(30)}</Text>
			</Box>

			<Box gap={0} marginLeft={1}>
				<Text color="green">{'  PROVIDER: '}</Text>
				{PROVIDERS.map((p, i) => (
					<Text key={p.id}>
						{i === providerIdx ? (
							<Text bold color="green">{`[${p.label}]`}</Text>
						) : (
							<Text color="gray">{` ${p.label} `}</Text>
						)}
						{i < PROVIDERS.length - 1 && <Text color="green">{' │ '}</Text>}
					</Text>
				))}
			</Box>

			<Box marginLeft={1}>
				<Text color="gray">{'  '}j/k to switch provider · r to run analysis</Text>
			</Box>

			{loading && (
				<Box gap={0} marginLeft={1}>
					<Text color="green">{'$ '}</Text>
					<Text color="green">{`analyzing logs with ${provider.label}...`}</Text>
					<Text color="green">{'█'}</Text>
				</Box>
			)}

			{result.status === 'done' && (
				<Box flexDirection="column" gap={0}>
					<Box gap={0} marginLeft={1}>
						<Text color="green">{'$ '}</Text>
						<Text color="green">{`analysis complete (${result.provider || provider.id})`}</Text>
						<Text color="gray">{' ✓'}</Text>
					</Box>
					<Box flexDirection="column" marginLeft={1} borderStyle="single" borderColor="green" paddingX={1}>
						{result.output.split('\n').map((line, i) => (
							<Box key={i} gap={0}>
								<Text color="green">{'│ '}</Text>
								<Text>{line}</Text>
							</Box>
						))}
					</Box>
				</Box>
			)}

			{result.status === 'error' && (
				<Box flexDirection="column" gap={0}>
					<Box gap={0} marginLeft={1}>
						<Text color="green">{'$ '}</Text>
						<Text color="red">{'[ERROR] analysis failed'}</Text>
					</Box>
					{result.error && (
						<Box marginLeft={1}>
							<Text color="red">{`  ${result.error}`}</Text>
						</Box>
					)}
					{result.output && (
						<Box flexDirection="column" marginLeft={1} borderStyle="single" borderColor="red" paddingX={1}>
							{result.output.split('\n').map((line, i) => (
								<Text key={i} color="red">{line}</Text>
							))}
						</Box>
					)}
				</Box>
			)}

			{result.status === 'idle' && !loading && (
				<Box marginLeft={1} gap={0}>
					<Text color="green">{'$ '}</Text>
					<Text color="gray">{'_ '}</Text>
					<Text color="gray" italic>{`press [r] to run with ${provider.label}`}</Text>
				</Box>
			)}
		</Box>
	);
}
