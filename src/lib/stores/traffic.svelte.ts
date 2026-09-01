import type { HttpExchange, ExtractedJwt, ProxyConfig, RouteRule, SavedRequestTemplate, TargetEnvironment, DiscoveredTarget, RelayProject } from "$lib/types";

const SAVED_PROJECTS_STORAGE_KEY = "relay_saved_projects_v1";
const ACTIVE_PROJECT_ID_KEY = "relay_active_project_id_v1";

function createDefaultProject(): RelayProject {
  return {
    id: "proj-default",
    name: "Projeto Padrão",
    description: "Espaço de trabalho padrão do Relay",
    config: {
      listenPort: 8080,
      targetHost: "127.0.0.1",
      targetPort: 3000,
      latencyMs: 0,
      jitterMs: 0,
      simulateFailureRate: 0.0,
      failureStatusCode: 500,
      autoExtractJwt: true,
      routes: [],
    },
    savedTemplates: [],
    savedEnvironments: [],
    createdAt: Date.now(),
  };
}

function loadProjectsFromStorage(): RelayProject[] {
  try {
    const raw = localStorage.getItem(SAVED_PROJECTS_STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) {
        return parsed;
      }
    }
  } catch (e) {
    // Ignora
  }
  return [createDefaultProject()];
}

function loadActiveProjectId(projects: RelayProject[]): string {
  try {
    const activeId = localStorage.getItem(ACTIVE_PROJECT_ID_KEY);
    if (activeId && projects.some(p => p.id === activeId)) {
      return activeId;
    }
  } catch (e) {
    // Ignora
  }
  return projects[0].id;
}

function saveProjectsToStorage(projects: RelayProject[]): void {
  try {
    localStorage.setItem(SAVED_PROJECTS_STORAGE_KEY, JSON.stringify(projects));
  } catch (e) {
    console.warn("Falha ao persistir projetos no storage:", e);
  }
}

class RelayState {
  exchanges = $state<HttpExchange[]>([]);
  selectedExchange = $state<HttpExchange | null>(null);
  
  // Comparação Diff entre 2 requisições
  diffCompareExchange = $state<HttpExchange | null>(null);

  // Sistema de Multi-Projetos
  projects = $state<RelayProject[]>(loadProjectsFromStorage());
  activeProjectId = $state<string>(loadActiveProjectId(this.projects));

  activeProject = $derived.by(() => {
    return this.projects.find(p => p.id === this.activeProjectId) || this.projects[0];
  });

  // Coleção de Requisições Salvas / Templates (vinculado ao projeto ativo)
  savedTemplates = $state<SavedRequestTemplate[]>([]);
  selectedTemplate = $state<SavedRequestTemplate | null>(null);

  // Navegação Lateral: "collection" (Padrão) vs "history" (Tráfego Real)
  sidebarTab = $state<"history" | "collection">("collection");

  // Ambientes Salvos (vinculado ao projeto ativo)
  savedEnvironments = $state<TargetEnvironment[]>([]);
  
  // Alvos Descobertos por Varredura Local de Portas
  discoveredTargets = $state<DiscoveredTarget[]>([]);
  isScanningTargets = $state<boolean>(false);

  // Ambiente Atualmente Ativo
  activeTarget = $state<TargetEnvironment | null>(null);

  // Variáveis capturadas dinamicamente das respostas (ex: id de customer, token, etc.)
  extractedVariables = $state<Record<string, string>>({});

  jwts = $state<ExtractedJwt[]>([]);
  selectedJwt = $state<ExtractedJwt | null>(null);
  isProxyRunning = $state<boolean>(false);
  activeView = $state<"traffic" | "jwt">("traffic");
  
  // Filtros de Tráfego
  searchQuery = $state<string>("");
  methodFilter = $state<string>("ALL");
  statusFilter = $state<string>("ALL");

  // Configuração do Proxy
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

  constructor() {
    this.syncCurrentProjectState();
  }

  syncCurrentProjectState(): void {
    const current = this.activeProject;
    if (current) {
      this.config = { ...current.config, routes: current.config.routes ? [...current.config.routes] : [] };
      this.savedTemplates = current.savedTemplates ? [...current.savedTemplates] : [];
      this.savedEnvironments = current.savedEnvironments ? [...current.savedEnvironments] : [];
      this.activeTarget = null;
    }
  }

  saveCurrentProject(): void {
    const idx = this.projects.findIndex(p => p.id === this.activeProjectId);
    if (idx >= 0) {
      this.projects[idx] = {
        ...this.projects[idx],
        config: { ...this.config },
        savedTemplates: [...this.savedTemplates],
        savedEnvironments: [...this.savedEnvironments],
      };
      saveProjectsToStorage(this.projects);
    }
  }

  // Ações de Projetos
  switchProject(projectId: string): void {
    if (projectId === this.activeProjectId) return;
    this.saveCurrentProject();
    this.activeProjectId = projectId;
    localStorage.setItem(ACTIVE_PROJECT_ID_KEY, projectId);
    this.syncCurrentProjectState();
    this.clear();
  }

  createProject(name: string, description?: string): void {
    this.saveCurrentProject();
    const newProj: RelayProject = {
      id: `proj-${Date.now()}`,
      name: name.trim() || "Novo Projeto",
      description: description?.trim(),
      config: {
        listenPort: 8080,
        targetHost: "127.0.0.1",
        targetPort: 3000,
        latencyMs: 0,
        jitterMs: 0,
        simulateFailureRate: 0.0,
        failureStatusCode: 500,
        autoExtractJwt: true,
        routes: [],
      },
      savedTemplates: [],
      savedEnvironments: [],
      createdAt: Date.now(),
    };
    this.projects = [...this.projects, newProj];
    this.activeProjectId = newProj.id;
    saveProjectsToStorage(this.projects);
    localStorage.setItem(ACTIVE_PROJECT_ID_KEY, newProj.id);
    this.syncCurrentProjectState();
    this.clear();
  }

  updateProject(id: string, name: string, description?: string): void {
    this.projects = this.projects.map(p => p.id === id ? { ...p, name: name.trim(), description: description?.trim() } : p);
    saveProjectsToStorage(this.projects);
  }

  deleteProject(id: string): void {
    if (this.projects.length <= 1) return;
    const remaining = this.projects.filter(p => p.id !== id);
    this.projects = remaining;
    if (this.activeProjectId === id) {
      this.activeProjectId = remaining[0].id;
      localStorage.setItem(ACTIVE_PROJECT_ID_KEY, remaining[0].id);
      this.syncCurrentProjectState();
      this.clear();
    }
    saveProjectsToStorage(this.projects);
  }

  // Derived - Variáveis Ativas
  activeVariables = $derived.by((): Record<string, string> => {
    return {
      baseUrl: `http://${this.config.targetHost}:${this.config.targetPort}`,
      ...this.extractedVariables,
    };
  });

  // Derived - Estatísticas
  totalRequests = $derived(this.exchanges.length);
  totalTemplates = $derived(this.savedTemplates.length);
  failedRequests = $derived(
    this.exchanges.filter(e => e.status === "failed" || (e.response && e.response.statusCode >= 400)).length
  );
  totalJwts = $derived(this.jwts.length);

  // Derived - Lista Filtrada do Histórico
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

  // Derived - Templates Filtrados
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

  // Helper para substituir variáveis {{varName}} no texto
  replaceVariables(text: string): string {
    let result = text;
    const vars = this.activeVariables;
    for (const [key, value] of Object.entries(vars)) {
      const pattern = new RegExp(`\\{\\{${key}\\}\\}`, "g");
      result = result.replace(pattern, value);
    }
    return result;
  }

  // Auto-Captura de Campos em Respostas JSON (id, customerId, accountId, token, etc.)
  extractVariablesFromResponse(bodyStr?: string): void {
    if (!bodyStr || !bodyStr.trim()) return;
    try {
      const parsed = JSON.parse(bodyStr);
      if (typeof parsed !== "object" || parsed === null) return;

      const newVars: Record<string, string> = { ...this.extractedVariables };

      const scanObj = (obj: Record<string, any>, prefix = "") => {
        for (const [k, v] of Object.entries(obj)) {
          if (typeof v === "string" || typeof v === "number") {
            const keyName = prefix ? `${prefix}_${k}` : k;
            newVars[keyName] = String(v);

            if (k === "id" && !prefix) {
              newVars["lastId"] = String(v);
              newVars["customerId"] = String(v);
            }
            if (k === "accountId" || k === "sourceAccountId") {
              newVars["accountId"] = String(v);
              newVars["sourceAccountId"] = String(v);
            }
            if (k === "access_token" || k === "token") {
              newVars["token"] = String(v);
            }
          } else if (typeof v === "object" && v !== null && !Array.isArray(v)) {
            scanObj(v, k);
          }
        }
      };

      scanObj(parsed);
      this.extractedVariables = newVars;
    } catch {
      // Body não é JSON válido
    }
  }

  // Persistência de Ambientes
  addSavedEnvironment(env: TargetEnvironment): void {
    this.savedEnvironments = [...this.savedEnvironments, env];
    this.saveCurrentProject();
  }

  updateSavedEnvironment(updated: TargetEnvironment): void {
    this.savedEnvironments = this.savedEnvironments.map(e => e.id === updated.id ? updated : e);
    this.saveCurrentProject();
    if (this.activeTarget?.id === updated.id) {
      this.selectTarget(updated);
    }
  }

  removeSavedEnvironment(id: string): void {
    this.savedEnvironments = this.savedEnvironments.filter(e => e.id !== id);
    this.saveCurrentProject();
    if (this.activeTarget?.id === id) {
      this.selectTarget(null);
    }
  }

  selectTarget(target: TargetEnvironment | null): void {
    this.activeTarget = target;
    if (target) {
      this.config.targetHost = target.host;
      this.config.targetPort = target.port;
      this.saveCurrentProject();
    }
  }

  // Persistência de Configuração do Proxy
  updateConfig(newConfig: ProxyConfig): void {
    this.config = newConfig;
    this.saveCurrentProject();
  }

  // Persistência de Templates / Coleções
  setTemplates(templates: SavedRequestTemplate[]): void {
    this.savedTemplates = templates;
    this.saveCurrentProject();
  }

  addTemplate(template: SavedRequestTemplate): void {
    this.savedTemplates = [...this.savedTemplates, template];
    this.saveCurrentProject();
  }

  clearTemplates(): void {
    this.savedTemplates = [];
    this.selectedTemplate = null;
    this.saveCurrentProject();
  }

  // Actions - Requisições
  addExchange(exchange: HttpExchange): void {
    this.exchanges = [exchange, ...this.exchanges];
  }

  updateResponse(requestId: string, response: HttpExchange["response"]): void {
    const item = this.exchanges.find(e => e.id === requestId);
    if (item) {
      item.response = response;
      item.status = "completed";
      if (response?.body) {
        this.extractVariablesFromResponse(response.body);
      }
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
    this.diffCompareExchange = null;
  }

  // Actions - JWTs
  addJwt(jwt: ExtractedJwt): void {
    const existingIndex = this.jwts.findIndex(j => j.token === jwt.token);
    if (existingIndex >= 0) {
      this.jwts[existingIndex] = jwt;
    } else {
      this.jwts = [jwt, ...this.jwts];
    }
    this.extractedVariables["token"] = jwt.token;
    if (jwt.subject) {
      this.extractedVariables["sub"] = jwt.subject;
      this.extractedVariables["customerId"] = jwt.subject;
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
    this.saveCurrentProject();
  }

  removeRoute(index: number): void {
    this.config.routes.splice(index, 1);
    this.saveCurrentProject();
  }
}

export const relayState = new RelayState();
