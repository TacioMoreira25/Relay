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

export interface SavedRequestTemplate {
  id: string;
  name: string;
  description?: string;
  tag?: string;
  method: HttpMethod;
  uri: string;
  headers: HeaderEntry[];
  body?: string;
  requiresAuth?: boolean;
}

export interface RouteRule {
  pathPrefix: string;
  targetHost?: string;
  targetPort: number;
  latencyMs?: number;
  isMock?: boolean;
  mockStatusCode?: number;
  mockBody?: string;
}

export interface TargetEnvironment {
  id: string;
  name: string;
  host: string;
  port: number;
  isHttps: boolean;
  isActive: boolean;
  type: "auto" | "saved" | "mock";
}

export interface DiscoveredTarget {
  id: string;
  label: string;
  host: string;
  port: number;
  isActive: boolean;
  source: "auto_discovered" | "manual" | "remote";
}

export interface ProxyConfig {
  listenPort: number;
  targetHost: string;
  targetPort: number;
  latencyMs: number;
  jitterMs: number;
  simulateFailureRate: number; // 0.0 to 1.0 (ex: 0.25 = 25%)
  failureStatusCode: number; // 500, 502, 503, 504
  autoExtractJwt: boolean;
  routes: RouteRule[];
}

export interface ExtractedJwt {
  token: string;
  source: string;
  detectedAt: number;
  claims?: Record<string, unknown>;
  header?: Record<string, unknown>;
  subject?: string;
  issuer?: string;
  expiresAt?: number;
}

export interface GeneratedCa {
  certPem: string;
  keyPem: string;
  commonName: string;
  createdAt: number;
}
