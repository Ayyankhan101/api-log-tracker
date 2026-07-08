import React from 'react';
import {Box, Text} from 'ink';
import type {DashboardStats} from '../types.js';

type Props = {
	stats: DashboardStats;
};

function PixelBar({value, max, width, color}: {value: number; max: number; width: number; color: string}) {
	const filled = max > 0 ? Math.round((value / max) * width) : 0;
	const empty = width - filled;
	return (
		<Text>
			<Text color={color}>{'█'.repeat(filled)}</Text>
			<Text color="gray">{'░'.repeat(empty)}</Text>
		</Text>
	);
}

function Sparkline({data}: {data: number[]}) {
	if (data.length === 0) {
		return <Text color="gray">{'░'.repeat(20)}</Text>;
	}

	const max = Math.max(...data, 1);
	const normalized = data.slice(-20).map(v => Math.min(Math.round((v / max) * 7), 7));

	const blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
	return (
		<Text color="green">
			{normalized.map(i => blocks[i]).join('')}
		</Text>
	);
}

function StatCard({label, value, color, suffix}: {label: string; value: string; color: string; suffix?: string}) {
	return (
		<Box flexDirection="column" width={16} borderStyle="single" borderColor="green" paddingX={1}>
			<Text color="gray" dimColor>{label}</Text>
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
				<StatCard label="TOTAL REQS" value={stats.total.toLocaleString()} color="green" />
				<StatCard label="ERROR RATE" value={stats.errorRate.toFixed(1)} color={stats.errorRate > 10 ? 'red' : stats.errorRate > 5 ? 'yellow' : 'green'} suffix="%" />
				<StatCard label="AVG LATENCY" value={stats.avgLatency.toFixed(0)} color="green" suffix="ms" />
				<StatCard label="MAX LATENCY" value={stats.maxLatency.toLocaleString()} color="red" suffix="ms" />
			</Box>

			<Box flexDirection="column" gap={0} marginTop={1}>
				<Text color="green">{'╔═ STATUS DISTRIBUTION ═════════════════════════════════════╗'}</Text>
				<Box gap={2} marginLeft={1}>
					<Text color="green">[2xx] </Text><Text color="green">{greenCount}</Text>
					<Text color="yellow">[4xx] </Text><Text color="yellow">{yellowCount}</Text>
					<Text color="red">[5xx] </Text><Text color="red">{redCount}</Text>
				</Box>
				{statusCounts.map(([code, count]) => {
					const color = code.startsWith('2') ? 'green' : code.startsWith('4') ? 'yellow' : 'red';
					return (
						<Box key={code} gap={1} marginLeft={1}>
							<Text color={color}>{code.padStart(4)}</Text>
							<PixelBar value={count} max={Math.max(...Object.values(stats.statusCodes), 1)} width={30} color={color} />
							<Text color="gray">
								{count} ({((count / total) * 100).toFixed(1)}%)
							</Text>
						</Box>
					);
				})}
				<Text color="green">{'╚═════════════════════════════════════════════════════════╝'}</Text>
			</Box>

			<Box flexDirection="column" gap={0} marginTop={1}>
				<Text color="green">{'╔═ TOP ENDPOINTS ═════════════════════════════════════════╗'}</Text>
				{endpoints.map(([endpoint, count]) => (
					<Box key={endpoint} gap={1} marginLeft={1}>
						<Text color="green">{'>'} </Text>
						<Text color="white">{endpoint.padEnd(40).slice(0, 40)}</Text>
						<PixelBar value={count} max={Math.max(...endpoints.map(e => e[1]), 1)} width={16} color="green" />
						<Text color="gray">{count}</Text>
					</Box>
				))}
				<Text color="green">{'╚═════════════════════════════════════════════════════════╝'}</Text>
			</Box>

			<Box flexDirection="column" gap={0} marginTop={1}>
				<Text color="green">{'╔═ REQUESTS/MIN (last 20 min) ════════════════════════════╗'}</Text>
				<Box marginLeft={1}>
					<Text color="green">{'  '}</Text>
					<Sparkline data={stats.requestsPerMinute} />
				</Box>
				<Text color="green">{'╚═════════════════════════════════════════════════════════╝'}</Text>
			</Box>
		</Box>
	);
}
