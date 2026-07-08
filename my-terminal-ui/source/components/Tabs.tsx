import React from 'react';
import {Box, Text} from 'ink';
import type {TabId} from '../types.js';

const TABS: {id: TabId; label: string; key: string}[] = [
	{id: 'dashboard', label: 'Dashboard', key: '1'},
	{id: 'logs', label: 'Live Logs', key: '2'},
	{id: 'analysis', label: 'Analysis', key: '3'},
	{id: 'controls', label: 'Controls', key: '4'},
	{id: 'integration', label: 'Integrate', key: '5'},
];

type Props = {
	active: TabId;
};

export default function Tabs({active}: Props) {
	return (
		<Box flexDirection="row" gap={1}>
			<Text color="green">{'['}</Text>
			{TABS.map((tab, i) => {
				const isActive = tab.id === active;
				return (
					<Text key={tab.id}>
						{i > 0 && <Text color="green">{'│'}</Text>}
						{isActive ? (
							<Text bold color="green">
								{` ${tab.key}:${tab.label} `}
							</Text>
						) : (
							<Text color="gray">{` ${tab.key}:${tab.label} `}</Text>
						)}
					</Text>
				);
			})}
			<Text color="green">{']'}</Text>
		</Box>
	);
}
