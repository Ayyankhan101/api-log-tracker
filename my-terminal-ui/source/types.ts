export type LogEntry = {
	id: string;
	timestamp: string;
	source: 'server' | 'client';
	method: string;
	endpoint: string;
	status_code: number;
	latency_ms: number;
	request_size: number;
	response_size: number;
	error: string | null;
};

export type TabId = 'dashboard' | 'logs' | 'analysis' | 'controls' | 'integration';

export type ServerStatus = 'stopped' | 'starting' | 'running' | 'error';

export type DashboardStats = {
	total: number;
	errors: number;
	errorRate: number;
	avgLatency: number;
	maxLatency: number;
	statusCodes: Record<number, number>;
	endpoints: Record<string, number>;
	requestsPerMinute: number[];
};

export type AnalysisResult = {
	status: 'idle' | 'running' | 'done' | 'error';
	output: string;
	error: string | null;
	provider?: string;
};
