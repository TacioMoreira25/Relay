import type { HttpExchange, ExtractedJwt, ProxyConfig } from "$lib/types";

class RelayState {
  exchanges = $state<HttpExchange[]>([]);
  selectedExchange = $state<HttpExchange | null>(null);
  jwts = $state<ExtractedJwt[]>([]);
  selectedJwt = $state<ExtractedJwt | null>(null);
  isProxyRunning = $state<boolean>(false);
  activeView = $state<"traffic" | "jwt">("traffic");
  
  // Filtros de Tráfego
  searchQuery = $state<string>("");
  methodFilter = $state<string>("ALL");
  statusFilter = $state<string>("ALL"); // "ALL", "2xx", "3xx", "4xx", "5xx", "ERR"

  config = $state<ProxyConfig>({
    listenPort: 8080,
    targetHost: "127.0.0.1",
    targetPort: 3000,
    latencyMs: 0,
    jitterMs: 0,
    simulateFailureRate: 0.0,
    failureStatusCode: 500,
    autoExtractJwt: true,
  });

  // Derived - Estatísticas
  totalRequests = $derived(this.exchanges.length);
  failedRequests = $derived(
    this.exchanges.filter(e => e.status === "failed" || (e.response && e.response.statusCode >= 400)).length
  );
  totalJwts = $derived(this.jwts.length);

  // Derived - Lista Filtrada em Tempo Real
  filteredExchanges = $derived(
    this.exchanges.filter(e => {
      if (this.methodFilter !== "ALL" && e.request.method.toUpperCase() !== this.methodFilter) {
        return false;
      }

      if (this.statusFilter !== "ALL") {
        if (this.statusFilter === "ERR") {
          if (e.status !== "failed" && (!e.response || e.response.statusCode < 400)) return false;
        } else if (this.statusFilter === "2xx") {
          if (!e.response || e.response.statusCode < 200 || e.response.statusCode >= 300) return false;
        } else if (this.statusFilter === "3xx") {
          if (!e.response || e.response.statusCode < 300 || e.response.statusCode >= 400) return false;
        } else if (this.statusFilter === "4xx") {
          if (!e.response || e.response.statusCode < 400 || e.response.statusCode >= 500) return false;
        } else if (this.statusFilter === "5xx") {
          if (!e.response || e.response.statusCode < 500) return false;
        }
      }

      if (this.searchQuery.trim()) {
        const query = this.searchQuery.toLowerCase().trim();
        const matchUri = e.request.uri.toLowerCase().includes(query);
        const matchMethod = e.request.method.toLowerCase().includes(query);
        const matchBody = e.request.body?.toLowerCase().includes(query) ?? false;
        const matchResBody = e.response?.body?.toLowerCase().includes(query) ?? false;
        return matchUri || matchMethod || matchBody || matchResBody;
      }

      return true;
    })
  );

  // Actions - Requisições
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

  // Actions - JWTs
  addJwt(jwt: ExtractedJwt): void {
    const existingIndex = this.jwts.findIndex(j => j.token === jwt.token);
    if (existingIndex >= 0) {
      this.jwts[existingIndex] = jwt;
    } else {
      this.jwts = [jwt, ...this.jwts];
    }
  }

  selectJwt(jwt: ExtractedJwt | null): void {
    this.selectedJwt = jwt;
  }

  clearJwts(): void {
    this.jwts = [];
    this.selectedJwt = null;
  }
}

export const relayState = new RelayState();
