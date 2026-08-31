import type { HttpExchange, ExtractedJwt, ProxyConfig, RouteRule, SavedRequestTemplate } from "$lib/types";

class RelayState {
  exchanges = $state<HttpExchange[]>([]);
  selectedExchange = $state<HttpExchange | null>(null);
  
  // Coleção de Requisições Salvas / Templates
  savedTemplates = $state<SavedRequestTemplate[]>([]);
  selectedTemplate = $state<SavedRequestTemplate | null>(null);

  // Navegação Lateral: "history" (Tráfego Real) vs "collection" (Rotas Salvas)
  sidebarTab = $state<"history" | "collection">("history");

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
    routes: [],
  });

  // Derived - Estatísticas
  totalRequests = $derived(this.exchanges.length);
  totalTemplates = $derived(this.savedTemplates.length);
  failedRequests = $derived(
    this.exchanges.filter(e => e.status === "failed" || (e.response && e.response.statusCode >= 400)).length
  );
  totalJwts = $derived(this.jwts.length);

  // Derived - Lista Filtrada do Histórico em Tempo Real
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

  // Derived - Templates Filtrados por busca
  filteredTemplates = $derived(
    this.savedTemplates.filter(t => {
      if (this.methodFilter !== "ALL" && t.method.toUpperCase() !== this.methodFilter) {
        return false;
      }
      if (this.searchQuery.trim()) {
        const query = this.searchQuery.toLowerCase().trim();
        return (
          t.name.toLowerCase().includes(query) ||
          t.uri.toLowerCase().includes(query) ||
          (t.tag && t.tag.toLowerCase().includes(query)) ||
          (t.body && t.body.toLowerCase().includes(query))
        );
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

  // Actions - Templates / Coleções
  setTemplates(templates: SavedRequestTemplate[]): void {
    this.savedTemplates = templates;
  }

  addTemplate(template: SavedRequestTemplate): void {
    this.savedTemplates = [...this.savedTemplates, template];
  }

  clearTemplates(): void {
    this.savedTemplates = [];
    this.selectedTemplate = null;
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

  // Actions - Rotas
  addRoute(route: RouteRule): void {
    this.config.routes.push(route);
  }

  removeRoute(index: number): void {
    this.config.routes.splice(index, 1);
  }
}

export const relayState = new RelayState();
