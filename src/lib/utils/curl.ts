import type { HeaderEntry, HttpExchange } from "$lib/types";

export interface CurlOptions {
  method: string;
  uri: string;
  headers?: HeaderEntry[];
  body?: string | null;
  baseUrl?: string;
}

/**
 * Gera um comando cURL pronto para terminal a partir de um HttpExchange ou objeto similar.
 */
export function generateCurl(
  target: HttpExchange | CurlOptions,
  defaultBaseUrl = "http://127.0.0.1:8080"
): string {
  const req = "request" in target ? target.request : target;
  const method = (req.method || "GET").toUpperCase();
  const uri = req.uri || "/";
  const baseUrl = ("baseUrl" in target && target.baseUrl) ? target.baseUrl : defaultBaseUrl;

  const fullUrl = uri.startsWith("http://") || uri.startsWith("https://")
    ? uri
    : `${baseUrl.replace(/\/$/, "")}${uri.startsWith("/") ? "" : "/"}${uri}`;

  let curl = `curl -i -X ${method} "${fullUrl}"`;

  if (req.headers && req.headers.length > 0) {
    for (const h of req.headers) {
      if (!h.key || h.key.trim().length === 0) continue;
      // Escapa aspas no valor do cabeçalho
      const escapedVal = h.value.replace(/"/g, '\\"');
      curl += ` \\\n  -H "${h.key}: ${escapedVal}"`;
    }
  }

  if (req.body && req.body.trim().length > 0) {
    const escapedBody = req.body.replace(/"/g, '\\"');
    curl += ` \\\n  -d "${escapedBody}"`;
  }

  return curl;
}
