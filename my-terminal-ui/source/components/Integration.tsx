import React, {useState} from 'react';
import {Box, Text, useInput} from 'ink';
import {theme} from './theme.js';

const LANGUAGES = [
	{id: 'python', label: 'Python'},
	{id: 'c', label: 'C'},
	{id: 'cpp', label: 'C++'},
	{id: 'rust', label: 'Rust'},
	{id: 'go', label: 'Go'},
	{id: 'erlang', label: 'Erlang'},
	{id: 'providers', label: 'LLM Providers'},
] as const;

type LanguageId = (typeof LANGUAGES)[number]['id'];

const SNIPPETS: Record<LanguageId, {title: string; lines: string[]}> = {
	python: {
		title: 'Python',
		lines: [
			'import requests, uuid, datetime',
			'',
			'def log_api_call(method, endpoint, status, latency_ms,',
			'                 source="my-app"):',
			'    requests.post("http://localhost:8080/api/log",',
			'        json={',
			'            "source": source,',
			'            "method": method,',
			'            "endpoint": endpoint,',
			'            "status_code": status,',
			'            "latency_ms": latency_ms,',
			'        }, timeout=1)',
			'',
			'# Usage',
			'log_api_call("GET", "https://api.example.com/data",',
			'             200, 145)',
		],
	},
	c: {
		title: 'C',
		lines: [
			'#include <curl/curl.h>',
			'',
			'void log_api_call(const char* method,',
			'                  const char* endpoint,',
			'                  int status, long latency_ms) {',
			'    CURL* curl = curl_easy_init();',
			'    char json[512];',
			'    snprintf(json, sizeof(json),',
			'        "{"source":"c-app","method":"%s",',
			'         "endpoint":"%s","status_code":%d,',
			'         "latency_ms":%ld}",',
			'        method, endpoint, status, latency_ms);',
			'    curl_easy_setopt(curl, CURLOPT_URL,',
			'        "http://localhost:8080/api/log");',
			'    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, json);',
			'    curl_easy_setopt(curl, CURLOPT_TIMEOUT_MS, 500);',
			'    curl_easy_perform(curl);',
			'    curl_easy_cleanup(curl);',
			'}',
		],
	},
	cpp: {
		title: 'C++',
		lines: [
			'#include <curl/curl.h>',
			'#include <string>',
			'#include <format>',
			'',
			'void log_api_call(const std::string& method,',
			'                  const std::string& endpoint,',
			'                  int status, long latency_ms) {',
			'    auto json = std::format(R"(',
			'        {{"source":"cpp-app","method":"{}",',
			'         "endpoint":"{}","status_code":{},',
			'         "latency_ms":{}}})",',
			'        method, endpoint, status, latency_ms);',
			'    CURL* curl = curl_easy_init();',
			'    curl_easy_setopt(curl, CURLOPT_URL,',
			'        "http://localhost:8080/api/log");',
			'    curl_easy_setopt(curl, CURLOPT_POSTFIELDS,',
			'        json.c_str());',
			'    curl_easy_setopt(curl, CURLOPT_TIMEOUT_MS, 500);',
			'    curl_easy_perform(curl);',
			'    curl_easy_cleanup(curl);',
			'}',
		],
	},
	rust: {
		title: 'Rust',
		lines: [
			'let client = reqwest::Client::new();',
			'client.post("http://localhost:8080/api/log")',
			'    .json(&serde_json::json!({',
			'        "source": "my-rust-app",',
			'        "method": method,',
			'        "endpoint": endpoint,',
			'        "status_code": status,',
			'        "latency_ms": latency,',
			'    }))',
			'    .timeout(std::time::Duration::from_millis(500))',
			'    .send().await.ok();',
		],
	},
	go: {
		title: 'Go',
		lines: [
			'import (',
			'    "bytes"',
			'    "encoding/json"',
			'    "net/http"',
			')',
			'',
			'func LogAPICall(method, endpoint string,',
			'               status int, latencyMs int64) {',
			'    body, _ := json.Marshal(map[string]any{',
			'        "source":      "go-app",',
			'        "method":      method,',
			'        "endpoint":    endpoint,',
			'        "status_code": status,',
			'        "latency_ms":  latencyMs,',
			'    })',
			'    http.Post(',
			'        "http://localhost:8080/api/log",',
			'        "application/json",',
			'        bytes.NewReader(body),',
			'    )',
			'}',
		],
	},
	erlang: {
		title: 'Erlang',
		lines: [
			'log_api_call(Method, Endpoint, Status, Latency) ->',
			'    Body = jsx:encode(#{',
			'        source => <<"erlang-app">>,',
			'        method => Method,',
			'        endpoint => Endpoint,',
			'        status_code => Status,',
			'        latency_ms => Latency',
			'    }),',
			'    httpc:request(',
			'        post,',
			'        {"http://localhost:8080/api/log",',
			'         [], "application/json", Body},',
			'        [], []',
			'    ).',
		],
	},
	providers: {
		title: 'LLM Providers (12 supported)',
		lines: [
			'# Set LLM_PROVIDER env var + the matching API key',
			'',
			'# OpenAI-compatible (same API, different key/url):',
			'export LLM_PROVIDER=openai      export OPENAI_API_KEY=sk-...',
			'export LLM_PROVIDER=grok        export XAI_API_KEY=...',
			'export LLM_PROVIDER=groq        export GROQ_API_KEY=...',
			'export LLM_PROVIDER=deepseek    export DEEPSEEK_API_KEY=...',
			'export LLM_PROVIDER=qwen        export DASHSCOPE_API_KEY=...',
			'export LLM_PROVIDER=baichuan    export BAICHUAN_API_KEY=...',
			'export LLM_PROVIDER=yi          export YI_API_KEY=...',
			'export LLM_PROVIDER=stepfun     export STEPFUN_API_KEY=...',
			'',
			'# Unique APIs:',
			'export LLM_PROVIDER=anthropic   export ANTHROPIC_API_KEY=sk-ant-...',
			'export LLM_PROVIDER=gemini      export GEMINI_API_KEY=...',
			'',
			'# Custom auth:',
			'export LLM_PROVIDER=glm         export ZHIPU_API_KEY=id.secret',
			'export LLM_PROVIDER=ernie       export BAIDU_API_KEY=... BAIDU_API_SECRET=...',
			'',
			'# Optional model override:',
			'export LLM_MODEL=gpt-4o',
		],
	},
};

export default function Integration() {
	const [selectedIdx, setSelectedIdx] = useState(0);

	useInput(input => {
		if (input === 'j' || input === 'ArrowDown') {
			setSelectedIdx(i => (i + 1) % LANGUAGES.length);
		}

		if (input === 'k' || input === 'ArrowUp') {
			setSelectedIdx(i => (i - 1 + LANGUAGES.length) % LANGUAGES.length);
		}
	});

	const selected = LANGUAGES[selectedIdx]!;
	const snippet = SNIPPETS[selected.id];

	return (
		<Box flexDirection="column" gap={1}>
			<Box gap={0}>
				<Text color={theme.primary}>{`╔═ INTEGRATION SNIPPETS ${theme.borderH.repeat(40)}╗`}</Text>
			</Box>

			<Box gap={1} marginLeft={1}>
				<Text color={theme.dim}>{'  '}</Text>
				{LANGUAGES.map((lang, i) => (
					<Text key={lang.id}>
						{i === selectedIdx ? (
							<Text bold color={theme.primary}>{`[${lang.label}]`}</Text>
						) : (
							<Text color={theme.dim}>{` ${lang.label} `}</Text>
						)}
						{i < LANGUAGES.length - 1 && <Text color={theme.primary}>{' │ '}</Text>}
					</Text>
				))}
				<Text color={theme.dim}>{'  (j/k to navigate)'}</Text>
			</Box>

			<Box marginLeft={1} marginTop={1} borderStyle="single" borderColor={theme.primary} paddingX={1}>
				<Text color={theme.primary}>
					{` ┌─ ${snippet.title} ─`}
				</Text>
			</Box>

			<Box flexDirection="column" marginLeft={2}>
				{snippet.lines.map((line, i) => (
					<Text key={i} color={line.startsWith('#') || line.startsWith('//') || line.startsWith('%') ? theme.dim : theme.bright}>
						{line}
					</Text>
				))}
			</Box>

			<Box marginLeft={1} marginTop={1}>
				<Text color={theme.primary}>{'└'}</Text>
				<Text color={theme.dim}>{'───────────────────────────────────────────────────────'}</Text>
				<Text color={theme.primary}>{'┘'}</Text>
			</Box>

			<Box flexDirection="column" gap={0} marginLeft={1} marginTop={1}>
				<Text color={theme.primary}>{'  Daemon: cargo run -- daemon'}</Text>
				<Text color={theme.dim}>{'  POST http://localhost:8080/api/log'}</Text>
				<Text color={theme.dim}>{'  POST http://localhost:8080/api/analyze'}</Text>
				<Text color={theme.dim}>{'  GET  http://localhost:8080/api/health'}</Text>
				<Text color={theme.dim}>{'  GET  http://localhost:8080/api/logs?limit=50'}</Text>
				<Text color={theme.dim}>{'  CSV:   logs/api_logs.csv'}</Text>
			</Box>
		</Box>
	);
}
