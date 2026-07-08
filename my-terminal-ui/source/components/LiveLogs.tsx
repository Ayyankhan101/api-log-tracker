import React, {useState} from 'react';
import {Box, Text, useInput} from 'ink';
import type {LogEntry} from '../types.js';

type Props = {
	entries: LogEntry[];
};

function formatTimestamp(ts: string): string {
	try {
		const date = new Date(ts);
		return date.toLocaleTimeString('en-US', {hour12: false});
	} catch {
		return ts.slice(11, 19);
	}
}

export default function LiveLogs({entries}: Props) {
	const [filter, setFilter] = useState<'all' | 'server' | 'client'>('all');
	const [scrollOffset, setScrollOffset] = useState(0);

	const filtered = filter === 'all' ? entries : entries.filter(e => e.source === filter);
	const sorted = [...filtered].reverse();
	const visibleCount = 20;
	const visible = sorted.slice(scrollOffset, scrollOffset + visibleCount);
	const maxScroll = Math.max(0, sorted.length - visibleCount);

	useInput(input => {
		if (input === 'f') {
			setFilter(filter === 'all' ? 'server' : filter === 'server' ? 'client' : 'all');
			setScrollOffset(0);
		}
	});

	return (
		<Box flexDirection="column" gap={0}>
			<Box gap={1} marginBottom={1}>
				<Text color="green">{'╔═ LIVE LOGS '}</Text>
				<Text color="gray">{`[${sorted.length} entries]`}</Text>
				<Text color="green">{' ═'}</Text>
				<Text color="gray">{' filter: '}</Text>
				<Text bold color="green">{filter}</Text>
				<Text color="gray">{' (f) '}</Text>
				{maxScroll > 0 && (
					<Text color="gray">{`scroll [${scrollOffset + 1}-${Math.min(scrollOffset + visibleCount, sorted.length)}/${sorted.length}]`}</Text>
				)}
				<Text color="green">{'═'.repeat(20)}</Text>
			</Box>

			<Box flexDirection="row" gap={0} marginLeft={1}>
				<Text bold color="green">{'  TIME     '}</Text>
				<Text bold color="green">{'SRC      '}</Text>
				<Text bold color="green">{'METHOD   '}</Text>
				<Text bold color="green">{'ENDPOINT                          '}</Text>
				<Text bold color="green">{'STS   '}</Text>
				<Text bold color="green">{'LATENCY'}</Text>
			</Box>
			<Text color="green">{'  ─'.repeat(42)}</Text>

			{visible.length === 0 && (
				<Text color="gray" italic>{'  No data. Waiting for signal...'}</Text>
			)}

			{visible.map((entry, i) => {
				const methodColor = entry.method === 'GET' ? 'green' : entry.method === 'POST' ? 'yellow' : 'cyan';
				const statusColor = entry.status_code < 300 ? 'green' : entry.status_code < 500 ? 'yellow' : 'red';
				const latencyColor = entry.latency_ms < 100 ? 'green' : entry.latency_ms < 500 ? 'yellow' : 'red';

				return (
					<Box key={`${entry.id}-${i}`} gap={0} marginLeft={1}>
						<Text color="green">{'> '}</Text>
						<Text color="gray">{formatTimestamp(entry.timestamp).padEnd(10)}</Text>
						<Text color={entry.source === 'server' ? 'green' : 'gray'}>
							{entry.source.padEnd(8)}
						</Text>
						<Text color={methodColor}>
							{entry.method.padEnd(8)}
						</Text>
						<Text color="white">
							{entry.endpoint.padEnd(40).slice(0, 40)}
						</Text>
						<Text color={statusColor}>[{entry.status_code}]</Text>
						<Text color={latencyColor}>{' '}{entry.latency_ms}ms</Text>
					</Box>
				);
			})}

			{sorted.length > visibleCount && (
				<Box marginTop={1}>
					<Text color="gray" dimColor>{'  ↑↓ to scroll'}</Text>
				</Box>
			)}
		</Box>
	);
}
