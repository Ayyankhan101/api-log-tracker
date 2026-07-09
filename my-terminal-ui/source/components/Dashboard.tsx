import React from 'react';
import {Box, Text} from 'ink';
import type {DashboardStats} from '../types.js';
import {theme} from './theme.js';

type Props = {
	stats: DashboardStats;
};

function PixelBar({value, max, width, color}: {value: number; max: number; width: number; color: string}) {
	const filled = max > 0 ? Math.round((value / max) * width) : 0;
	const empty = width - filled;
	return (
		<Text>
			<Text color={color}>{theme.filled.repeat(filled)}</Text>
			<Text color={theme.dim}>{theme.empty.repeat(empty)}</Text>
		</Text>
	);
}

function Sparkline({data}: {data: number[]}) {
	if (data.length === 0) {
		return <Text color={theme.dim}>{theme.empty.repeat(20)}</Text>;
	}

	const max = Math.max(...data, 1);
	const normalized = data.slice(-20).map(v => Math.min(Math.round((v / max) * 7), 7));

	const blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
	return (
		<Text color={theme.primary}>
			{normalized.map(i => blocks[i]).join('')}
		</Text>
	);
}

function StatCard({label, value, color, suffix}: {label: string; value: string; color: string; suffix?: string}) {
	return (
		<Box flexDirection="column" width={16} borderStyle="single" borderColor={theme.primary} paddingX={1}>
			<Text color={theme.dim} dimColor>{label}</Text>
			<Text bold color={color}>
				{value}{suffix || ''}
			</Text>
		</Box>
	);
}

export default function Dashboard({stats}: Props) {
	const statusCounts = Object.entries(stats.statusCodes);
	const total = stats.total || 1;
	const endpoints = Object.entries(stats.endpoints)
		.sort((a, b) => b[1] - a[1])
		.slice(0, 5);

	const greenCount = statusCounts
		.filter(([code]) => code.startsWith('2'))
		.reduce((sum, [, count]) => sum + count, 0);
	const yellowCount = statusCounts
		.filter(([code]) => code.startsWith('4'))
		.reduce((sum, [, count]) => sum + count, 0);
	const redCount = statusCounts
		.filter(([code]) => code.startsWith('5'))
		.reduce((sum, [, count]) => sum + count, 0);

	return (
		<Box flexDirection="column" gap={1}>
			<Box flexDirection="row" gap={1}>
				<StatCard label="TOTAL REQS" value={stats.total.toLocaleString()} color={theme.primary} />
				<StatCard label="ERROR RATE" value={stats.errorRate.toFixed(1)} color={stats.errorRate > 10 ? theme.error : stats.errorRate > 5 ? theme.warn : theme.primary} suffix="%" />
				<StatCard label="AVG LATENCY" value={stats.avgLatency.toFixed(0)} color={theme.primary} suffix="ms" />
				<StatCard label="MAX LATENCY" value={stats.maxLatency.toLocaleString()} color={theme.error} suffix="ms" />
			</Box>

			<Box flexDirection="column" gap={0} marginTop={1}>
				<Text color={theme.primary}>{`╔═ STATUS DISTRIBUTION ${theme.borderH.repeat(46)}╗`}</Text>
				<Box gap={2} marginLeft={1}>
					<Text color={theme.primary}>[2xx] </Text><Text color={theme.primary}>{greenCount}</Text>
					<Text color={theme.warn}>[4xx] </Text><Text color={theme.warn}>{yellowCount}</Text>
					<Text color={theme.error}>[5xx] </Text><Text color={theme.error}>{redCount}</Text>
				</Box>
				{statusCounts.map(([code, count]) => {
					const color = code.startsWith('2') ? theme.primary : code.startsWith('4') ? theme.warn : theme.error;
					return (
						<Box key={code} gap={1} marginLeft={1}>
							<Text color={color}>{code.padStart(4)}</Text>
							<PixelBar value={count} max={Math.max(...Object.values(stats.statusCodes), 1)} width={30} color={color} />
							<Text color={theme.dim}>
								{count} ({((count / total) * 100).toFixed(1)}%)
							</Text>
						</Box>
					);
				})}
				<Text color={theme.primary}>{`╚${theme.borderH.repeat(55)}╝`}</Text>
			</Box>

			<Box flexDirection="column" gap={0} marginTop={1}>
				<Text color={theme.primary}>{`╔═ TOP ENDPOINTS ${theme.borderH.repeat(44)}╗`}</Text>
				{endpoints.map(([endpoint, count]) => (
					<Box key={endpoint} gap={1} marginLeft={1}>
						<Text color={theme.primary}>{'>'} </Text>
						<Text color={theme.bright}>{endpoint.padEnd(40).slice(0, 40)}</Text>
						<PixelBar value={count} max={Math.max(...endpoints.map(e => e[1]), 1)} width={16} color={theme.primary} />
						<Text color={theme.dim}>{count}</Text>
					</Box>
				))}
				<Text color={theme.primary}>{`╚${theme.borderH.repeat(55)}╝`}</Text>
			</Box>

			<Box flexDirection="column" gap={0} marginTop={1}>
				<Text color={theme.primary}>{`╔═ REQUESTS/MIN (last 20 min) ${theme.borderH.repeat(33)}╗`}</Text>
				<Box marginLeft={1}>
					<Text color={theme.primary}>{'  '}</Text>
					<Sparkline data={stats.requestsPerMinute} />
				</Box>
				<Text color={theme.primary}>{`╚${theme.borderH.repeat(55)}╝`}</Text>
			</Box>
		</Box>
	);
}
