/**
 * Classes CSS e estilos padronizados para Métodos HTTP e Status Codes.
 */

export function getMethodBadgeClass(method?: string): string {
  if (!method) return "bg-zinc-800 text-zinc-300 border-zinc-700";

  switch (method.toUpperCase()) {
    case "GET":
      return "text-cyan-300 bg-cyan-500/15 border-cyan-500/40";
    case "POST":
      return "text-emerald-300 bg-emerald-500/15 border-emerald-500/40";
    case "PUT":
      return "text-amber-300 bg-amber-500/15 border-amber-500/40";
    case "PATCH":
      return "text-purple-300 bg-purple-500/15 border-purple-500/40";
    case "DELETE":
      return "text-rose-300 bg-rose-500/15 border-rose-500/40";
    case "OPTIONS":
      return "text-sky-300 bg-sky-500/15 border-sky-500/40";
    case "HEAD":
      return "text-indigo-300 bg-indigo-500/15 border-indigo-500/40";
    default:
      return "bg-zinc-800 text-zinc-300 border-zinc-700";
  }
}

export function getStatusCodeClass(code?: number, statusStr?: string): string {
  if (statusStr === "pending") return "text-zinc-500 italic";
  if (statusStr === "failed" || (!code && statusStr)) return "text-rose-400 font-bold";
  if (!code) return "text-zinc-400";

  if (code >= 200 && code < 300) return "text-emerald-400 font-semibold";
  if (code >= 300 && code < 400) return "text-cyan-400 font-semibold";
  if (code >= 400 && code < 500) return "text-amber-400 font-semibold";
  if (code >= 500) return "text-rose-400 font-bold";

  return "text-zinc-400";
}

export function getStatusCodeBadgeClass(code?: number): string {
  if (!code) return "bg-zinc-800 text-zinc-400 border-zinc-700";

  if (code >= 200 && code < 300) {
    return "bg-emerald-500/15 text-emerald-300 border-emerald-500/30";
  }
  if (code >= 300 && code < 400) {
    return "bg-cyan-500/15 text-cyan-300 border-cyan-500/30";
  }
  if (code >= 400 && code < 500) {
    return "bg-amber-500/15 text-amber-300 border-amber-500/30";
  }
  if (code >= 500) {
    return "bg-rose-500/15 text-rose-300 border-rose-500/30";
  }

  return "bg-zinc-800 text-zinc-400 border-zinc-700";
}
