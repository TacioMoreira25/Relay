# Chaos Engineering & Simulador de Resiliência de Rede

O módulo de Chaos Engineering do Relay permite simular cenários adversos de conectividade em tempo real sem alterar uma única linha de código do frontend ou do backend.

---

## 1. Injeção de Latência com Jitter

Permite validar o comportamento de telas de carregamento, spinners, skeletons e timeouts de clientes HTTP.

- **Latência Base (`latency_ms`)**: Atraso mínimo adicionado a cada requisição antes de repassar ao upstream.
- **Jitter Aleatório (`jitter_ms`)**: Variação aleatória aplicada sobre a latência base para emular redes móveis oscilantes (3G/4G/Wi-Fi instável).
- **Fórmula de Cálculo**:
  $$\text{Delay Total} = \text{latency\_ms} + \text{rand}(0, \text{jitter\_ms})$$

---

## 2. Injeção Controlada de Falhas (Failure Rates)

Permite testar como o frontend lida com indisponibilidade de serviços e respostas de erro da API.

- **Taxa de Falhas (`simulate_failure_rate`)**: Percentual de 0% a 100% de requisições que devem ser abortadas com erro simulado.
- **Códigos HTTP Suportados**:
  - `500 Internal Server Error`
  - `502 Bad Gateway`
  - `503 Service Unavailable`
  - `504 Gateway Timeout`
  - `429 Too Many Requests`
- **Rastreabilidade**: Toda resposta gerada pelo motor de Chaos inclui o cabeçalho `x-relay-chaos: simulated-failure` e payload JSON detalhado com a duração do atraso.
