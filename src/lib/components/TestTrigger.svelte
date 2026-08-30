<script lang="ts">
  import { relayState } from "$lib/stores/traffic.svelte";

  let isSending = $state(false);
  let logs = $state<string[]>([]);

  async function sendMockExchange(): Promise<void> {
    isSending = true;
    const now = Date.now();
    const reqId = "test-" + Math.random().toString(36).substring(2, 9);

    // Simula evento relay:request
    relayState.addExchange({
      id: reqId,
      request: {
        id: reqId,
        timestamp: now,
        method: "POST",
        uri: "/api/v1/auth/login",
        headers: [
          { key: "Host", value: "api.relay.internal" },
          { key: "Content-Type", value: "application/json" },
          { key: "User-Agent", value: "Relay-Desktop-TestClient/1.0" },
          { key: "Authorization", value: "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IlRhY2lvIiwicm9sZSI6ImRldmVsb3BlciIsImlhdCI6MTUxNjIzOTAyMn0.9-pY8L7e_8kE1qM2P4z" }
        ],
        body: JSON.stringify({
          username: "tacio",
          environment: "Fedora 44 / KDE Plasma",
          action: "test_proxy_inspection"
        }, null, 2),
        sizeBytes: 128,
      },
      status: "pending"
    });

    // Simula latência de resposta de 42ms
    await new Promise((r) => setTimeout(r, 42));

    // Simula evento relay:response
    relayState.updateResponse(reqId, {
      id: "res-" + Math.random().toString(36).substring(2, 9),
      requestId: reqId,
      timestamp: Date.now(),
      statusCode: 200,
      headers: [
        { key: "Content-Type", value: "application/json" },
        { key: "X-Server", value: "Relay-Tokio-Engine" },
        { key: "X-Response-Time", value: "42ms" }
      ],
      body: JSON.stringify({
        status: "authenticated",
        user: {
          id: 1,
          name: "Tacio",
          role: "admin"
        },
        token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
        expiresIn: 3600
      }, null, 2),
      sizeBytes: 256,
      durationMs: 42,
    });

    isSending = false;
  }
</script>

<div class="flex items-center space-x-2">
  <button
    onclick={sendMockExchange}
    disabled={isSending}
    class="text-xs px-3 py-1.5 rounded font-medium bg-emerald-600/20 text-emerald-300 border border-emerald-500/40 hover:bg-emerald-600/30 transition-all flex items-center space-x-1.5 active:scale-95 cursor-pointer"
  >
    <span>⚡</span>
    <span>{isSending ? "Injetando..." : "Testar Requisição"}</span>
  </button>
</div>
