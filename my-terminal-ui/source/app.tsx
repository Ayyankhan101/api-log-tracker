import React, {useEffect, useMemo, useState} from 'react';
import {Box, Text, useInput, useApp} from 'ink';
import Tabs from './components/Tabs.js';
import Dashboard from './components/Dashboard.js';
import LiveLogs from './components/LiveLogs.js';
import Analysis from './components/Analysis.js';
import ServerControls from './components/ServerControls.js';
import Integration from './components/Integration.js';
import {createCsvWatcher} from './csv-watcher.js';
import {theme} from './components/theme.js';
import type {LogEntry, TabId, ServerStatus, DashboardStats, AnalysisResult} from './types.js';

type Props = {
	csvPath: string;
};

function computeStats(entries: LogEntry[]): DashboardStats {
	const total = entries.length;
	const errors = entries.filter(e => e.error !== null).length;
	const errorRate = total > 0 ? (errors / total) * 100 : 0;
	const avgLatency = total > 0 ? entries.reduce((sum, e) => sum + e.latency_ms, 0) / total : 0;
	const maxLatency = entries.length > 0 ? Math.max(...entries.map(e => e.latency_ms)) : 0;

	const statusCodes: Record<number, number> = {};
	const endpoints: Record<string, number> = {};
	for (const entry of entries) {
		statusCodes[entry.status_code] = (statusCodes[entry.status_code] || 0) + 1;
		endpoints[entry.endpoint] = (endpoints[entry.endpoint] || 0) + 1;
	}

	const now = Date.now();
	const requestsPerMinute: number[] = [];
	for (let i = 19; i >= 0; i--) {
		const start = now - (i + 1) * 60_000;
		const end = now - i * 60_000;
		const count = entries.filter(e => {
			const ts = new Date(e.timestamp).getTime();
			return ts >= start && ts < end;
		}).length;
		requestsPerMinute.push(count);
	}

	return {total, errors, errorRate, avgLatency, maxLatency, statusCodes, endpoints, requestsPerMinute};
}

function Header({entries, serverStatus}: {entries: LogEntry[]; serverStatus: ServerStatus}) {
	const statusColor = serverStatus === 'running' ? theme.primary : theme.dim;
	const dot = serverStatus === 'running' ? '●' : '○';

	return (
		<Box flexDirection="column" marginBottom={1}>
			<Text color={theme.primary}>
				{`┌─[api-log-tracker]─[v0.1.2]${theme.borderH.repeat(34)}┐`}
			</Text>
			<Text color={theme.primary}>
				{`${theme.borderV}  ${theme.cursor}`}
				<Text bold color={theme.primary}>{theme.filled}</Text>
				{'  '}
				<Text color={theme.dim}>{String(entries.length).padStart(4)} entries</Text>
				{'  '}
				<Text color={statusColor}>{dot} {serverStatus === 'running' ? 'ONLINE' : 'OFFLINE'}</Text>
				{'                                │'}
			</Text>
			<Text color={theme.primary}>
				{`└${theme.borderH.repeat(56)}┘`}
			</Text>
		</Box>
	);
}

function Footer() {
	return (
		<Box marginTop={1} gap={1}>
			<Text color={theme.primary}>{theme.filled} </Text>
			<Text color={theme.dim}>{`1-5`}</Text>
			<Text color={theme.primary}>{' switch'}</Text>
			<Text color={theme.dim}>{' │ '}</Text>
			<Text color={theme.dim}>{`q`}</Text>
			<Text color={theme.primary}>{' quit'}</Text>
		</Box>
	);
}

const MAX_ENTRIES = 100_000;

export default function App({csvPath}: Props) {
	const {exit} = useApp();
	const [activeTab, setActiveTab] = useState<TabId>('dashboard');
	const [entries, setEntries] = useState<LogEntry[]>([]);
	const [serverStatus, setServerStatus] = useState<ServerStatus>('stopped');
	const [analysisResult, setAnalysisResult] = useState<AnalysisResult>({status: 'idle', output: '', error: null});
	const [showHelp, setShowHelp] = useState(false);

	useEffect(() => {
		const watcher = createCsvWatcher(csvPath, {
			onEntry(entry) {
				setEntries(prev => {
					const next = [...prev, entry];
					return next.length > MAX_ENTRIES ? next.slice(-MAX_ENTRIES) : next;
				});
			},
			onBatch(batch) {
				setEntries(prev => {
					const next = [...prev, ...batch];
					return next.length > MAX_ENTRIES ? next.slice(-MAX_ENTRIES) : next;
				});
			},
		});

		return () => {
			watcher.close();
		};
	}, [csvPath]);

	const stats = useMemo(() => computeStats(entries), [entries]);

	useInput((input, key) => {
		if (input === 'q' || (key.ctrl && input === 'c')) {
			exit();
		}

		if (input === '?') {
			setShowHelp(prev => !prev);
			return;
		}

		if (showHelp) {
			setShowHelp(false);
			return;
		}

		if (input === '1') setActiveTab('dashboard');
		if (input === '2') setActiveTab('logs');
		if (input === '3') setActiveTab('analysis');
		if (input === '4') setActiveTab('controls');
		if (input === '5') setActiveTab('integration');
	});

	if (showHelp) {
		return (
			<Box flexDirection="column" padding={1}>
				<Text color={theme.primary} bold>{'╔═ KEYBOARD SHORTCUTS ════════════════════════════════════╗'}</Text>
				<Text color={theme.primary}>{'║'}</Text>
				<Text color={theme.primary}>{'║  GLOBAL'}</Text>
				<Text color={theme.primary}>{'║'}</Text>
				<Text color={theme.dim}>{'║    1-5        '}</Text><Text>Switch tabs</Text>
				<Text color={theme.dim}>{'║    q / Ctrl+C '}</Text><Text>Quit</Text>
				<Text color={theme.dim}>{'║    ?          '}</Text><Text>Toggle this help</Text>
				<Text color={theme.primary}>{'║'}</Text>
				<Text color={theme.primary}>{'║  DASHBOARD (tab 1)'}</Text>
				<Text color={theme.dim}>{'║    (auto-updates from CSV)'}</Text>
				<Text color={theme.primary}>{'║'}</Text>
				<Text color={theme.primary}>{'║  LIVE LOGS (tab 2)'}</Text>
				<Text color={theme.dim}>{'║    f          '}</Text><Text>Filter: all → server → client</Text>
				<Text color={theme.dim}>{'║    ↑ / ↓     '}</Text><Text>Scroll entries</Text>
				<Text color={theme.primary}>{'║'}</Text>
				<Text color={theme.primary}>{'║  ANALYSIS (tab 3)'}</Text>
				<Text color={theme.dim}>{'║    j / k      '}</Text><Text>Switch provider</Text>
				<Text color={theme.dim}>{'║    r          '}</Text><Text>Run analysis</Text>
				<Text color={theme.primary}>{'║'}</Text>
				<Text color={theme.primary}>{'║  SERVER CONTROLS (tab 4)'}</Text>
				<Text color={theme.dim}>{'║    s          '}</Text><Text>Start daemon</Text>
				<Text color={theme.dim}>{'║    k          '}</Text><Text>Stop daemon</Text>
				<Text color={theme.dim}>{'║    d          '}</Text><Text>Run demo client</Text>
				<Text color={theme.dim}>{'║    c          '}</Text><Text>Clear output</Text>
				<Text color={theme.primary}>{'║'}</Text>
				<Text color={theme.primary}>{'╚═════════════════════════════════════════════════════════╝'}</Text>
				<Text color={theme.dim} italic>{'  Press any key to close'}</Text>
			</Box>
		);
	}

	return (
		<Box flexDirection="column" height="100%">
			<Header entries={entries} serverStatus={serverStatus} />

			<Box marginBottom={1}>
				<Tabs active={activeTab} />
			</Box>

			<Box key={activeTab} flexDirection="column" flexGrow={1}>
				{activeTab === 'dashboard' && <Dashboard stats={stats} />}
				{activeTab === 'logs' && <LiveLogs entries={entries} />}
			{activeTab === 'analysis' && (
				<Analysis
					result={analysisResult}
					onResult={setAnalysisResult}
				/>
			)}
				{activeTab === 'controls' && <ServerControls serverStatus={serverStatus} onStatusChange={setServerStatus} />}
				{activeTab === 'integration' && <Integration />}
			</Box>

			<Footer />
		</Box>
	);
}
