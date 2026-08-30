export type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS";

export interface HeaderEntry {
  key: string;
  value: string;
}

export interface InterceptedRequest {
  id: string;
  timestamp: number;
  method: HttpMethod;
  uri: string;
  headers: HeaderEntry[];
  body?: string;
  sizeBytes: number;
}

export interface InterceptedResponse {
  id: string;
  requestId: string;
  timestamp: number;
  statusCode: number;
  headers: HeaderEntry[];
  body?: string;
  sizeBytes: number;
  durationMs: number;
}

export interface HttpExchange {
  id: string;
  request: InterceptedRequest;
  response?: InterceptedResponse;
  status: "pending" | "completed" | "failed";
  error?: string;
}

export interface ProxyConfig {
  listenPort: number;
  targetHost: string;
  targetPort: number;
  latencyMs: number;
  simulateFailureRate: number; // 0.0 to 1.0
  autoExtractJwt: boolean;
}

export interface ExtractedJwt {
  token: string;
  source: "request_header" | "response_header" | "response_body";
  detectedAt: number;
  claims?: Record<string, unknown>;
}
