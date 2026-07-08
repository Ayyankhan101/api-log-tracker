import React, {useEffect, useMemo, useState} from 'react';
import {Box, Text, useInput, useApp} from 'ink';
import Tabs from './components/Tabs.js';
import Dashboard from './components/Dashboard.js';
import LiveLogs from './components/LiveLogs.js';
import Analysis from './components/Analysis.js';
import ServerControls from './components/ServerControls.js';
import Integration from './components/Integration.js';
import {createCsvWatcher} from './csv-watcher.js';
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
	const statusColor = serverStatus === 'running' ? 'green' : 'gray';
	const dot = serverStatus === 'running' ? '●' : '○';

	return (
		<Box flexDirection="column" marginBottom={1}>
			<Text color="green">
				{'┌─[api-log-tracker]─[v0.1]──────────────────────────────────┐'}
			</Text>
			<Text color="green">
				{'│  $'}
				<Text bold color="green">{'█'}</Text>
				{'  '}
				<Text color="gray">{String(entries.length).padStart(4)} entries</Text>
				{'  '}
				<Text color={statusColor}>{dot} {serverStatus === 'running' ? 'ONLINE' : 'OFFLINE'}</Text>
				{'                                │'}
			</Text>
			<Text color="green">
				{'└────────────────────────────────────────────────────────────┘'}
			</Text>
		</Box>
	);
}

function Footer() {
	return (
		<Box marginTop={1} gap={1}>
			<Text color="green">{'█ '}</Text>
			<Text color="gray">{`1-5`}</Text>
			<Text color="green">{' switch'}</Text>
			<Text color="gray">{' │ '}</Text>
			<Text color="gray">{`q`}</Text>
			<Text color="green">{' quit'}</Text>
		</Box>
	);
}

export default function App({csvPath}: Props) {
	const {exit} = useApp();
	const [activeTab, setActiveTab] = useState<TabId>('dashboard');
	const [entries, setEntries] = useState<LogEntry[]>([]);
	const [serverStatus, setServerStatus] = useState<ServerStatus>('stopped');
	const [analysisResult, setAnalysisResult] = useState<AnalysisResult>({status: 'idle', output: '', error: null});

	useEffect(() => {
		const watcher = createCsvWatcher(csvPath, {
			onEntry(entry) {
				setEntries(prev => [...prev, entry]);
			},
			onBatch(batch) {
				setEntries(prev => [...prev, ...batch]);
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

		if (input === '1') setActiveTab('dashboard');
		if (input === '2') setActiveTab('logs');
		if (input === '3') setActiveTab('analysis');
		if (input === '4') setActiveTab('controls');
		if (input === '5') setActiveTab('integration');
	});

	return (
		<Box flexDirection="column" height="100%">
			<Header entries={entries} serverStatus={serverStatus} />

			<Box marginBottom={1}>
				<Tabs active={activeTab} />
			</Box>

			<Box flexDirection="column" flexGrow={1}>
				{activeTab === 'dashboard' && <Dashboard stats={stats} />}
				{activeTab === 'logs' && <LiveLogs entries={entries} />}
				{activeTab === 'analysis' && (
					<Analysis
						result={analysisResult}
						apiKey={process.env['ANTHROPIC_API_KEY'] || ''}
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
