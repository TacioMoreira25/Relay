# Funcionalidade: Cliente HTTP & Replay de Chamadas

## 1. Visão Geral
Permite repetir qualquer requisição previamente capturada ou criar novas requisições do zero no Relay, com possibilidade de modificar método, headers, query params ou JSON body antes do reenvio.

## 2. Casos de Uso e Recursos
* **Foco Automático na Resposta:** Ao disparar a chamada, o Relay seleciona a nova requisição no histórico e posiciona a visualização automaticamente na aba **Response**, permitindo conferência imediata do resultado sem cliques adicionais.
* **Identificação Visual no Histórico:** Toda chamada originada pelo Replay recebe o badge `MANUAL` em destaque no histórico.
* **Filtro Rápido:** Você pode usar o filtro `Manuais` na barra lateral para isolar exclusivamente as requisições geradas pelo Replay.
* **Auto-Injeção de JWT e Variáveis:** Substituição automática de variáveis dinâmicas no cabeçalho ou body (`{{token}}`, `{{sub}}`, `{{customerId}}`).
* **Salvar como Template na Coleção:** Salva a requisição formatada diretamente na Coleção do projeto ativo.
* **Captura de Resposta e Sessão:** Se a resposta do Replay contiver um token JWT (no body ou headers), o token é automaticamente decodificado e atualizado na sessão do Relay.

## 3. Componentes
* `ReplayModal.svelte`: Modal de disparo com suporte a edição de métodos, URI, injeção de variáveis, cabeçalhos dinâmicos e salvamento na coleção.
* `HeaderEditor.svelte`: Adiciona, edita e remove headers dinamicamente com inputs vinculados reativamente via `$bindable()`.
