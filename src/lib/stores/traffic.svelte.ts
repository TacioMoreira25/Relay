import type {
  ExtractedJwt,
  HttpExchange,
  ProxyConfig,
  SavedRequestTemplate,
  TargetEnvironment,
  DiscoveredTarget,
} from "$lib/types";

export interface ProjectData {
  id: string;
  name: string;
  description?: string;
  createdAt: number;
  config: ProxyConfig;
  savedTemplates: SavedRequestTemplate[];
  savedEnvironments: TargetEnvironment[];
}

const STORAGE_PROJECTS_KEY = "relay_projects_data";
const STORAGE_ACTIVE_PROJECT_KEY = "relay_active_project_id";

function loadProjectsFromStorage(): ProjectData[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_PROJECTS_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) return parsed;
    }
  } catch (e) {
    console.error("Falha ao carregar projetos do LocalStorage:", e);
  }

  const defaultProj: ProjectData = {
    id: "proj-default",
    name: "Projeto Padrão",
    description: "Ambiente padrão de monitoramento e testes",
    createdAt: Date.now(),
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
    savedEnvironments: [
      {
        id: "env-local-3000",
        name: "Localhost :3000",
        host: "127.0.0.1",
        port: 3000,
        isHttps: false,
        isActive: true,
        type: "saved",
      },
    ],
  };

  return [defaultProj];
}

function saveProjectsToStorage(projects: ProjectData[]): void {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem(STORAGE_PROJECTS_KEY, JSON.stringify(projects));
  } catch (e) {
    console.error("Falha ao salvar projetos:", e);
  }
}

class RelayState {
  projects = $state<ProjectData[]>(loadProjectsFromStorage());
  activeProjectId = $state<string>(
    typeof window !== "undefined"
      ? localStorage.getItem(STORAGE_ACTIVE_PROJECT_KEY) || "proj-default"
      : "proj-default"
  );

  exchanges = $state<HttpExchange[]>([]);
  selectedExchange = $state<HttpExchange | null>(null);
  diffCompareExchange = $state<HttpExchange | null>(null);

  savedTemplates = $state<SavedRequestTemplate[]>([]);
  selectedTemplate = $state<SavedRequestTemplate | null>(null);

  sidebarTab = $state<"history" | "collection">("collection");
  inspectorTab = $state<"request" | "response" | "diff" | "curl">("request");

  savedEnvironments = $state<TargetEnvironment[]>([]);
  discoveredTargets = $state<DiscoveredTarget[]>([]);
  isScanningTargets = $state<boolean>(false);
  activeTarget = $state<TargetEnvironment | null>(null);

  extractedVariables = $state<Record<string, string>>({});
  jwts = $state<ExtractedJwt[]>([]);
  selectedJwt = $state<ExtractedJwt | null>(null);
  isProxyRunning = $state<boolean>(false);
  activeView = $state<"traffic" | "jwt">("traffic");
  
  searchQuery = $state<string>("");
  methodFilter = $state<string>("ALL");
  statusFilter = $state<string>("ALL");

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
    this.loadActiveProject();
  }

  activeProject = $derived.by((): ProjectData => {
    return this.projects.find((p) => p.id === this.activeProjectId) || this.projects[0];
  });

  loadActiveProject(): void {
    const proj = this.projects.find((p) => p.id === this.activeProjectId) || this.projects[0];
    if (proj) {
      this.activeProjectId = proj.id;
      this.config = { ...proj.config };
      this.savedTemplates = proj.savedTemplates ? [...proj.savedTemplates] : [];
      this.savedEnvironments = proj.savedEnvironments ? [...proj.savedEnvironments] : [];
      this.activeTarget = this.savedEnvironments.find((e) => e.isActive) || this.savedEnvironments[0] || null;
      if (typeof window !== "undefined") {
        localStorage.setItem(STORAGE_ACTIVE_PROJECT_KEY, proj.id);
      }
    }
  }

  saveCurrentProject(): void {
    const idx = this.projects.findIndex((p) => p.id === this.activeProjectId);
    if (idx >= 0) {
      this.projects[idx].config = { ...this.config };
      this.projects[idx].savedTemplates = [...this.savedTemplates];
      this.projects[idx].savedEnvironments = [...this.savedEnvironments];
      saveProjectsToStorage(this.projects);
    }
  }

  createProject(name: string, description?: string): string {
    const newId = `proj-${Date.now()}`;
    const newProj: ProjectData = {
      id: newId,
      name: name.trim() || "Novo Projeto",
      description: description?.trim() || "",
      createdAt: Date.now(),
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
      savedEnvironments: [
        {
          id: `env-${Date.now()}`,
          name: "Localhost :3000",
          host: "127.0.0.1",
          port: 3000,
          isHttps: false,
          isActive: true,
          type: "saved",
        },
      ],
    };

    this.projects = [...this.projects, newProj];
    saveProjectsToStorage(this.projects);
    this.switchProject(newId);
    return newId;
  }

  updateProject(id: string, name: string, description?: string): void {
    const proj = this.projects.find(p => p.id === id);
    if (proj) {
      proj.name = name.trim();
      proj.description = description?.trim() || "";
      saveProjectsToStorage(this.projects);
    }
  }

  switchProject(id: string): void {
    this.saveCurrentProject();
    this.activeProjectId = id;
    this.loadActiveProject();
    this.clear();
  }

  deleteProject(id: string): void {
    if (this.projects.length <= 1) return;
    this.projects = this.projects.filter((p) => p.id !== id);
    if (this.activeProjectId === id) {
      this.activeProjectId = this.projects[0].id;
      this.loadActiveProject();
      this.clear();
    }
    saveProjectsToStorage(this.projects);
  }

  activeVariables = $derived.by((): Record<string, string> => {
    return {
      baseUrl: `http://${this.config.targetHost}:${this.config.targetPort}`,
      ...this.extractedVariables,
    };
  });

  totalRequests = $derived(this.exchanges.length);
  totalTemplates = $derived(this.savedTemplates.length);
  failedRequests = $derived(
    this.exchanges.filter(e => e.status === "failed" || (e.response && e.response.statusCode >= 400)).length
  );
  totalJwts = $derived(this.jwts.length);

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

  replaceVariables(text: string): string {
    let result = text;
    const vars = this.activeVariables;
    for (const [key, value] of Object.entries(vars)) {
      const pattern = new RegExp(`\\{\\{${key}\\}\\}`, "g");
      result = result.replace(pattern, value);
    }
    return result;
  }

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

  updateConfig(newConfig: ProxyConfig): void {
    this.config = newConfig;
    this.saveCurrentProject();
  }

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

  addExchange(exchange: HttpExchange): void {
    this.exchanges = [exchange, ...this.exchanges];
    if (!this.selectedExchange) {
      this.selectedExchange = exchange;
    }
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
    this.inspectorTab = "request";
  }

  clear(): void {
    this.exchanges = [];
    this.selectedExchange = null;
    this.diffCompareExchange = null;
    this.inspectorTab = "request";
  }

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
}

export const relayState = new RelayState();
