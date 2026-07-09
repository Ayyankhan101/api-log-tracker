import React from 'react';
import {Box, Text} from 'ink';
import type {TabId} from '../types.js';
import {theme} from './theme.js';

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
			<Text color={theme.primary}>{'['}</Text>
			{TABS.map((tab, i) => {
				const isActive = tab.id === active;
				return (
					<Text key={tab.id}>
						{i > 0 && <Text color={theme.primary}>{'│'}</Text>}
						{isActive ? (
							<Text bold color={theme.primary}>
								{` ${tab.key}:${tab.label} `}
							</Text>
						) : (
							<Text color={theme.dim}>{` ${tab.key}:${tab.label} `}</Text>
						)}
					</Text>
				);
			})}
			<Text color={theme.primary}>{']'}</Text>
		</Box>
	);
}
