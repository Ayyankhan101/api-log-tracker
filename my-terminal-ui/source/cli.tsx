#!/usr/bin/env node
import React from 'react';
import {render} from 'ink';
import meow from 'meow';
import App from './app.js';

const cli = meow(
	`
	Usage
	  $ api-log-tui

	Options
		--csv  Path to the CSV log file (default: ../logs/api_logs.csv)

	Examples
	  $ api-log-tui
	  $ api-log-tui --csv=./my-logs.csv
`,
	{
		importMeta: import.meta,
		flags: {
			csv: {
				type: 'string',
				default: '../logs/api_logs.csv',
			},
		},
	},
);

render(<App csvPath={cli.flags.csv} />);
