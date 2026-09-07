import { invoke } from "@tauri-apps/api/core";
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
const STORAGE_EXCHANGES_KEY = "relay_exchanges_history";
const STORAGE_JWTS_KEY = "relay_saved_jwts";

function loadJwtsFromStorage(): ExtractedJwt[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_JWTS_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed)) return parsed;
    }
  } catch (e) {
    console.error("Falha ao carregar JWTs do LocalStorage:", e);
  }
  return [];
}

function saveJwtsToStorage(jwts: ExtractedJwt[]): void {
  if (typeof window === "undefined") return;
  try {
    const limited = jwts.slice(0, 50);
    localStorage.setItem(STORAGE_JWTS_KEY, JSON.stringify(limited));
  } catch (e) {
    console.error("Falha ao salvar JWTs:", e);
  }
}

export function parseJwtClientSide(tokenStr: string, source: string): ExtractedJwt | null {
  try {
    const clean = tokenStr.trim().replace(/^(Bearer|bearer|BEARER)\s+/i, "").replace(/["'`]/g, "").trim();
    const parts = clean.split(".");
    if (parts.length !== 3) return null;

    let base64 = parts[1].replace(/-/g, "+").replace(/_/g, "/");
    while (base64.length % 4) {
      base64 += "=";
    }
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split("")
        .map((c) => "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2))
        .join("")
    );
    const claims = JSON.parse(jsonPayload);

    let header: Record<string, unknown> | undefined;
    try {
      let hB64 = parts[0].replace(/-/g, "+").replace(/_/g, "/");
      while (hB64.length % 4) hB64 += "=";
      header = JSON.parse(atob(hB64));
    } catch {
      // Ignora erro no header
    }

    return {
      token: clean,
      source,
      detectedAt: Date.now(),
      claims,
      header,
      subject: claims.sub ? String(claims.sub) : undefined,
      issuer: claims.iss ? String(claims.iss) : undefined,
      expiresAt: typeof claims.exp === "number" ? claims.exp : undefined,
    };
  } catch {
    return null;
  }
}

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
        id: "env-default",
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

function loadExchangesFromStorage(): HttpExchange[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_EXCHANGES_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed)) return parsed;
    }
  } catch (e) {
    console.error("Falha ao carregar histórico:", e);
  }
  return [];
}

function saveExchangesToStorage(exchanges: HttpExchange[]): void {
  if (typeof window === "undefined") return;
  try {
    // Manter no máximo os últimos 100 itens para não estourar o LocalStorage
    const limited = exchanges.slice(0, 100);
    localStorage.setItem(STORAGE_EXCHANGES_KEY, JSON.stringify(limited));
  } catch (e) {
    console.error("Falha ao salvar histórico:", e);
  }
}

class RelayState {
  projects = $state<ProjectData[]>(loadProjectsFromStorage());
  activeProjectId = $state<string>(
    typeof window !== "undefined"
      ? localStorage.getItem(STORAGE_ACTIVE_PROJECT_KEY) || "proj-default"
      : "proj-default"
  );

  exchanges = $state<HttpExchange[]>(loadExchangesFromStorage());
  selectedExchange = $state<HttpExchange | null>(null);
  diffCompareExchange = $state<HttpExchange | null>(null);

  savedTemplates = $state<SavedRequestTemplate[]>([]);
  selectedTemplate = $state<SavedRequestTemplate | null>(null);

  sidebarTab = $state<"history" | "collection">("collection");
  inspectorTab = $state<"request" | "response" | "diff" | "curl">("response");

  savedEnvironments = $state<TargetEnvironment[]>([]);
  discoveredTargets = $state<DiscoveredTarget[]>([]);
  isScanningTargets = $state<boolean>(false);
  activeTarget = $state<TargetEnvironment | null>(null);

  extractedVariables = $state<Record<string, string>>({});
  jwts = $state<ExtractedJwt[]>(loadJwtsFromStorage());
  selectedJwt = $state<ExtractedJwt | null>(null);
  isProxyRunning = $state<boolean>(false);
  activeView = $state<"traffic" | "jwt">("traffic");
  
  searchQuery = $state<string>("");
  methodFilter = $state<string>("ALL");
  statusFilter = $state<string>("ALL");
  historySourceFilter = $state<"ALL" | "MANUAL" | "AUTO">("ALL");
  hidePolling = $state<boolean>(false);

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

  createProject(name: string, description?: string, listenPort?: number): string {
    const newId = `proj-${Date.now()}`;
    const newProj: ProjectData = {
      id: newId,
      name: name.trim() || "Novo Projeto",
      description: description?.trim() || "",
      createdAt: Date.now(),
      config: {
        listenPort: listenPort && listenPort > 0 ? listenPort : 8080,
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

  updateProject(id: string, name: string, description?: string, listenPort?: number): void {
    const proj = this.projects.find(p => p.id === id);
    if (proj) {
      proj.name = name.trim();
      proj.description = description?.trim() || "";
      if (listenPort && listenPort > 0) {
        proj.config.listenPort = listenPort;
        if (this.activeProjectId === id) {
          this.config.listenPort = listenPort;
        }
      }
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

  filteredExchanges = $derived.by((): HttpExchange[] => {
    const list = this.exchanges;

    return list.filter((e, idx) => {
      // 1. Filtro por Método HTTP
      if (this.methodFilter !== "ALL" && e.request.method.toUpperCase() !== this.methodFilter) {
        return false;
      }

      // 2. Filtro por Status HTTP
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

      // 3. Filtro por Origem (Manual Replay vs Automático Proxy)
      const isManual = e.id.startsWith("replay-");
      if (this.historySourceFilter === "MANUAL" && !isManual) return false;
      if (this.historySourceFilter === "AUTO" && isManual) return false;

      // 4. Filtro: Ocultar Polling / Repetições Rápidas em Sequência
      if (this.hidePolling && !isManual) {
        // Se houver outra requisição mais recente idêntica em menos de 4s, oculta a anterior
        const firstMatchIndex = list.findIndex(
          other =>
            other.request.method === e.request.method &&
            other.request.uri === e.request.uri &&
            Math.abs(other.request.timestamp - e.request.timestamp) < 4000
        );
        if (firstMatchIndex >= 0 && firstMatchIndex !== idx) {
          return false;
        }
      }

      // 5. Filtro por Busca Textual
      if (this.searchQuery.trim()) {
        const query = this.searchQuery.toLowerCase().trim();
        const matchUri = e.request.uri.toLowerCase().includes(query);
        const matchMethod = e.request.method.toLowerCase().includes(query);
        const matchBody = e.request.body?.toLowerCase().includes(query) ?? false;
        const matchResBody = e.response?.body?.toLowerCase().includes(query) ?? false;
        return matchUri || matchMethod || matchBody || matchResBody;
      }

      return true;
    });
  });

  isPollingExchange(exchange: HttpExchange): boolean {
    if (exchange.id.startsWith("replay-")) return false;
    const sameRoute = this.exchanges.filter(
      e =>
        e.id !== exchange.id &&
        e.request.method === exchange.request.method &&
        e.request.uri === exchange.request.uri &&
        Math.abs(e.request.timestamp - exchange.request.timestamp) < 4000
    );
    return sameRoute.length > 0;
  }

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

  saveTemplate(tpl: SavedRequestTemplate): void {
    const idx = this.savedTemplates.findIndex(t => t.id === tpl.id);
    if (idx >= 0) {
      this.savedTemplates[idx] = tpl;
      this.savedTemplates = [...this.savedTemplates]; // trigger reactivity
    } else {
      this.savedTemplates = [...this.savedTemplates, tpl];
    }
    this.saveCurrentProject();
  }

  deleteTemplate(id: string): void {
    const idx = this.savedTemplates.findIndex(t => t.id === id);
    if (idx >= 0) {
      this.savedTemplates.splice(idx, 1);
      this.savedTemplates = [...this.savedTemplates]; // trigger reactivity
      this.saveCurrentProject();
    }
  }

  clearTemplates(): void {
    this.savedTemplates = [];
    this.selectedTemplate = null;
    this.saveCurrentProject();
  }

  addExchange(exchange: HttpExchange): void {
    const exists = this.exchanges.some(e => e.id === exchange.id);
    if (!exists) {
      this.exchanges = [exchange, ...this.exchanges];
    } else {
      this.exchanges = this.exchanges.map(e => (e.id === exchange.id ? exchange : e));
    }
    if (exchange.id.startsWith("replay-") || !this.selectedExchange) {
      this.selectedExchange = exchange;
      this.inspectorTab = "response";
    }
    this.scanForJwts(exchange);
    saveExchangesToStorage(this.exchanges);
  }

  updateResponse(requestId: string, response: HttpExchange["response"]): void {
    const item = this.exchanges.find(e => e.id === requestId);
    if (item) {
      item.response = response;
      item.status = "completed";
      if (response?.body) {
        this.extractVariablesFromResponse(response.body);
      }
      this.scanForJwts(item);
      saveExchangesToStorage(this.exchanges);

      // Sempre que chegar resposta (especialmente chamadas manuais), seleciona e abre na aba response
      if (this.selectedExchange?.id === requestId || requestId.startsWith("replay-")) {
        this.selectedExchange = item;
        this.inspectorTab = "response";
      }
    }
  }

  saveExchangeAsTemplate(exchange: HttpExchange, customName?: string): SavedRequestTemplate {
    const uriPath = exchange.request.uri.split("?")[0];
    const segments = uriPath.split("/").filter(Boolean);
    let derivedTag: string | undefined = undefined;
    if (segments.length > 0) {
      if (segments[0] === "api" && segments.length > 1) {
        derivedTag = segments[1];
      } else {
        derivedTag = segments[0];
      }
    }

    const templateName =
      customName || `${exchange.request.method} ${segments[segments.length - 1] || uriPath}`;

    const newTemplate: SavedRequestTemplate = {
      id: `tpl-${Date.now()}-${Math.random().toString(36).substring(2, 6)}`,
      name: templateName,
      description: `Salvo do histórico em ${new Date(exchange.request.timestamp).toLocaleString()}`,
      tag: derivedTag,
      method: exchange.request.method,
      uri: exchange.request.uri,
      headers: exchange.request.headers.filter(
        h => !["host", "content-length", "connection"].includes(h.key.toLowerCase())
      ),
      body: exchange.request.body,
      requiresAuth: exchange.request.headers.some(h =>
        ["authorization", "cookie", "x-access-token"].includes(h.key.toLowerCase())
      ),
    };

    this.addTemplate(newTemplate);
    return newTemplate;
  }

  select(exchange: HttpExchange | null): void {
    this.selectedExchange = exchange;
    this.inspectorTab = "response";
  }

  private scanForJwts(exchange: HttpExchange): void {
    // 1. Headers da Requisição
    for (const h of exchange.request.headers) {
      const keyLower = h.key.toLowerCase();
      if (keyLower === "authorization" || keyLower.includes("token") || keyLower === "cookie") {
        if (h.value.includes("ey")) {
          const jwt = parseJwtClientSide(h.value, `client_req_${keyLower}`);
          if (jwt) this.addJwt(jwt);
        }
      }
    }

    // 2. Headers e Body da Resposta
    if (exchange.response) {
      for (const h of exchange.response.headers) {
        const keyLower = h.key.toLowerCase();
        if (keyLower === "authorization" || keyLower.includes("token") || keyLower === "set-cookie") {
          if (h.value.includes("ey")) {
            const jwt = parseJwtClientSide(h.value, `client_res_${keyLower}`);
            if (jwt) this.addJwt(jwt);
          }
        }
      }

      if (exchange.response.body && exchange.response.body.includes("ey")) {
        try {
          const parsed = JSON.parse(exchange.response.body);
          const scanObj = (obj: any, keyName = "") => {
            if (typeof obj === "string") {
              if (obj.includes("ey")) {
                const jwt = parseJwtClientSide(obj, `client_body_${keyName || 'token'}`);
                if (jwt) this.addJwt(jwt);
              }
            } else if (typeof obj === "object" && obj !== null) {
              for (const [k, v] of Object.entries(obj)) {
                scanObj(v, k);
              }
            }
          };
          scanObj(parsed);
        } catch {
          const words = exchange.response.body.split(/\s+|[,"';]+/);
          for (const w of words) {
            if (w.startsWith("ey") && w.split(".").length === 3) {
              const jwt = parseJwtClientSide(w, "client_raw_body");
              if (jwt) this.addJwt(jwt);
            }
          }
        }
      }
    }
  }

  setError(requestId: string, errorMsg: string): void {
    const item = this.exchanges.find(e => e.id === requestId);
    if (item) {
      item.status = "failed";
      item.error = errorMsg;
      saveExchangesToStorage(this.exchanges);
    }
  }

  removeExchange(id: string): void {
    this.exchanges = this.exchanges.filter(e => e.id !== id);
    if (this.selectedExchange?.id === id) {
      this.selectedExchange = this.exchanges[0] || null;
    }
    saveExchangesToStorage(this.exchanges);
    try {
      invoke("delete_exchange", { id }).catch(e => console.warn("Erro ao deletar exchange no Rust:", e));
    } catch {
      // Ignora erro de invoke se fora de runtime Tauri
    }
  }

  removeExchanges(ids: string[]): void {
    if (!ids || ids.length === 0) return;
    const idSet = new Set(ids);
    this.exchanges = this.exchanges.filter(e => !idSet.has(e.id));
    if (this.selectedExchange && idSet.has(this.selectedExchange.id)) {
      this.selectedExchange = this.exchanges[0] || null;
    }
    saveExchangesToStorage(this.exchanges);
    try {
      invoke("delete_exchanges", { ids }).catch(e => console.warn("Erro ao deletar exchanges no Rust:", e));
    } catch {
      // Ignora erro de invoke se fora de runtime Tauri
    }
  }

  clear(): void {
    this.exchanges = [];
    this.selectedExchange = null;
    this.diffCompareExchange = null;
    this.inspectorTab = "request";
    saveExchangesToStorage(this.exchanges);
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
    saveJwtsToStorage(this.jwts);
  }

  selectJwt(jwt: ExtractedJwt | null): void {
    this.selectedJwt = jwt;
  }

  clearJwts(): void {
    this.jwts = [];
    this.selectedJwt = null;
    saveJwtsToStorage(this.jwts);
  }
}

export const relayState = new RelayState();
