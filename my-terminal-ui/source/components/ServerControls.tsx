import React, {useEffect, useState} from 'react';
import {Box, Text, useInput} from 'ink';
import type {ServerStatus} from '../types.js';
import {startServer, runDemoClient} from '../rust-bridge.js';

type Props = {
	serverStatus: ServerStatus;
	onStatusChange: (status: ServerStatus) => void;
};

export default function ServerControls({serverStatus, onStatusChange}: Props) {
	const [logs, setLogs] = useState<string[]>([]);
	const [demoRunning, setDemoRunning] = useState(false);
	const [serverProcess, setServerProcess] = useState<ReturnType<typeof startServer> | null>(null);

	useEffect(() => {
		return () => {
			serverProcess?.kill();
		};
	}, [serverProcess]);

	useInput((input, _key) => {
		if (input === 's' && serverStatus === 'stopped') {
			onStatusChange('starting');
			setLogs(prev => [...prev, '$ starting server...']);

			const proc = startServer({
				onStdout(data) {
					setLogs(prev => [...prev, data.trim()]);
					if (data.includes('listening')) {
						onStatusChange('running');
					}
				},
				onStderr(data) {
					setLogs(prev => [...prev, `$ [err] ${data.trim()}`]);
				},
				onExit(code) {
					onStatusChange('stopped');
					setLogs(prev => [...prev, `$ server exited (code ${code})`]);
					setServerProcess(null);
				},
				onError(error) {
					onStatusChange('error');
					setLogs(prev => [...prev, `$ [error] ${error.message}`]);
					setServerProcess(null);
				},
			});

			setServerProcess(proc);
		}

		if (input === 'k' && (serverStatus === 'running' || serverStatus === 'starting')) {
			serverProcess?.kill();
			setLogs(prev => [...prev, '$ server stopped.']);
		}

		if (input === 'd' && !demoRunning && serverStatus === 'running') {
			setDemoRunning(true);
			setLogs(prev => [...prev, '$ running demo client...']);

			runDemoClient({
				onStdout(data) {
					setLogs(prev => [...prev, data.trim()]);
				},
				onStderr(data) {
					setLogs(prev => [...prev, `$ [err] ${data.trim()}`]);
				},
				onExit(code) {
					setDemoRunning(false);
					setLogs(prev => [...prev, `$ demo done (code ${code})`]);
				},
				onError(error) {
					setDemoRunning(false);
					setLogs(prev => [...prev, `$ [error] ${error.message}`]);
				},
			});
		}

		if (input === 'c') {
			setLogs([]);
		}
	});

	const statusColor =
		serverStatus === 'running' ? 'green' : serverStatus === 'starting' ? 'yellow' : serverStatus === 'error' ? 'red' : 'gray';
	const statusIcon = serverStatus === 'running' ? '●' : serverStatus === 'starting' ? '◌' : '○';

	return (
		<Box flexDirection="column" gap={1}>
			<Box gap={0}>
				<Text color="green">{'╔═ SERVER CONTROLS ════════════════════════════════════════╗'}</Text>
			</Box>

			<Box gap={0} marginLeft={1}>
				<Text color="green">{'║  '}</Text>
				<Text color="gray">{'STATUS: '}</Text>
				<Text color={statusColor} bold>{statusIcon} [{serverStatus.toUpperCase()}]</Text>
				<Text color="green">{'                                       '}</Text>
				<Text color="green">{'║'}</Text>
			</Box>

			<Box flexDirection="column" gap={0} marginLeft={1} marginTop={1}>
				{serverStatus === 'stopped' && (
					<Text color="green">{'  '}[s] start server</Text>
				)}
				{(serverStatus === 'running' || serverStatus === 'starting') && (
					<Text color="red">{'  '}[k] stop server</Text>
				)}
				{serverStatus === 'running' && (
					<Text color="green">{'  '}[d] run demo client</Text>
				)}
				<Text color="gray">{'  '}[c] clear output</Text>
			</Box>

			{logs.length > 0 && (
				<Box flexDirection="column" gap={0} marginTop={1} marginLeft={1} borderStyle="single" borderColor="green" paddingX={1} height={15}>
					{logs.slice(-12).map((line, i) => (
						<Text key={i} color={line.startsWith('$') ? 'green' : 'gray'}>
							{line}
						</Text>
					))}
					<Text color="green">{'$ █'}</Text>
				</Box>
			)}
		</Box>
	);
}
