import React, {useState} from 'react';
import {Box, Text, useInput} from 'ink';
import type {LogEntry} from '../types.js';
import {theme} from './theme.js';

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

	useInput((input, key) => {
		if (input === 'f') {
			setFilter(filter === 'all' ? 'server' : filter === 'server' ? 'client' : 'all');
			setScrollOffset(0);
		}

		if (key.upArrow) {
			setScrollOffset(offset => Math.min(offset + 1, maxScroll));
		}

		if (key.downArrow) {
			setScrollOffset(offset => Math.max(offset - 1, 0));
		}
	});

	return (
		<Box flexDirection="column" gap={0}>
			<Box gap={1} marginBottom={1}>
				<Text color={theme.primary}>{`╔═ LIVE LOGS `}</Text>
				<Text color={theme.dim}>{`[${sorted.length} entries]`}</Text>
				<Text color={theme.primary}>{' ═'}</Text>
				<Text color={theme.dim}>{' filter: '}</Text>
				<Text bold color={theme.primary}>{filter}</Text>
				<Text color={theme.dim}>{' (f) '}</Text>
				{maxScroll > 0 && (
					<Text color={theme.dim}>{`scroll [${scrollOffset + 1}-${Math.min(scrollOffset + visibleCount, sorted.length)}/${sorted.length}]`}</Text>
				)}
				<Text color={theme.primary}>{theme.borderH.repeat(20)}</Text>
			</Box>

			<Box flexDirection="row" gap={0} marginLeft={1}>
				<Text bold color={theme.primary}>{'  TIME     '}</Text>
				<Text bold color={theme.primary}>{'SRC      '}</Text>
				<Text bold color={theme.primary}>{'METHOD   '}</Text>
				<Text bold color={theme.primary}>{'ENDPOINT                          '}</Text>
				<Text bold color={theme.primary}>{'STS   '}</Text>
				<Text bold color={theme.primary}>{'LATENCY'}</Text>
			</Box>
			<Text color={theme.primary}>{'  ─'.repeat(42)}</Text>

			{visible.length === 0 && (
				<Text color={theme.dim} italic>{'  No data. Waiting for signal...'}</Text>
			)}

			{visible.map((entry, i) => {
				const methodColor = entry.method === 'GET' ? theme.primary : entry.method === 'POST' ? theme.warn : theme.accent;
				const statusColor = entry.status_code < 300 ? theme.primary : entry.status_code < 500 ? theme.warn : theme.error;
				const latencyColor = entry.latency_ms < 100 ? theme.primary : entry.latency_ms < 500 ? theme.warn : theme.error;

				return (
					<Box key={`${entry.id}-${i}`} gap={0} marginLeft={1}>
						<Text color={theme.primary}>{'> '}</Text>
						<Text color={theme.dim}>{formatTimestamp(entry.timestamp).padEnd(10)}</Text>
						<Text color={entry.source === 'server' ? theme.primary : theme.dim}>
							{entry.source.padEnd(8)}
						</Text>
						<Text color={methodColor}>
							{entry.method.padEnd(8)}
						</Text>
						<Text color={theme.bright}>
							{entry.endpoint.padEnd(40).slice(0, 40)}
						</Text>
						<Text color={statusColor}>[{entry.status_code}]</Text>
						<Text color={latencyColor}>{' '}{entry.latency_ms}ms</Text>
					</Box>
				);
			})}

			{sorted.length > visibleCount && (
				<Box marginTop={1}>
					<Text color={theme.dim} dimColor>{'  ↑↓ to scroll'}</Text>
				</Box>
			)}
		</Box>
	);
}
