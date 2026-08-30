import type { HttpExchange, ExtractedJwt, ProxyConfig } from "$lib/types";

class RelayState {
  exchanges = $state<HttpExchange[]>([]);
  selectedExchange = $state<HttpExchange | null>(null);
  jwts = $state<ExtractedJwt[]>([]);
  isProxyRunning = $state<boolean>(false);
  
  config = $state<ProxyConfig>({
    listenPort: 8080,
    targetHost: "127.0.0.1",
    targetPort: 3000,
    latencyMs: 0,
    simulateFailureRate: 0.0,
    autoExtractJwt: true,
  });

  // Derived
  totalRequests = $derived(this.exchanges.length);
  failedRequests = $derived(this.exchanges.filter(e => e.status === "failed" || (e.response && e.response.statusCode >= 400)).length);

  // Actions
  addExchange(exchange: HttpExchange): void {
    this.exchanges = [exchange, ...this.exchanges];
  }

  updateResponse(requestId: string, response: HttpExchange["response"]): void {
    const item = this.exchanges.find(e => e.id === requestId);
    if (item) {
      item.response = response;
      item.status = "completed";
    }
  }

  setError(requestId: string, errorMsg: string): void {
    const item = this.exchanges.find(e => e.id === requestId);
    if (item) {
      item.status = "failed";
      item.error = errorMsg;
    }
  }

  select(exchange: HttpExchange | null): void {
    this.selectedExchange = exchange;
  }

  clear(): void {
    this.exchanges = [];
    this.selectedExchange = null;
  }
}

export const relayState = new RelayState();
