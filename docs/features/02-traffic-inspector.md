# Funcionalidade: Inspecionador de Tráfego em Tempo Real

## 1. Visão Geral
O Inspecionador exibe de forma reativa e sem latência perceptível todas as requisições que transitam pelo proxy ou são disparadas manualmente, divididas em uma visualização mestre-detalhe (**Request List** e **Inspector Panel**), com suporte a divisor redimensionável.

## 2. Componentes e Responsabilidades

### `RequestList.svelte`
* **Barra Lateral com Largura Móvel:** Divisor interativo (*drag handle*) que permite redimensionar a lista entre 260px e 750px, com salvamento automático no `localStorage`.
* **Lista Ordenada Cronologicamente:** As requisições mais recentes aparecem no topo com badges de método (`GET`, `POST`, `PUT`, `DELETE`), badges de origem (`MANUAL` para Replay ou `POLL` para repetições em intervalo curto), URI, código de status e tempo de execução (`ms`).
* **Filtros por Origem:** Segmentação imediata entre:
  * **Todas:** Exibe todo o tráfego interceptado.
  * **Manuais:** Isola apenas as requisições disparadas pelo Replay/Disparador.
  * **Capturadas:** Exibe apenas o tráfego gerado pela aplicação cliente/navegador.
* **Filtro Anti-Ruído ("Ocultar Polling"):** Oculta requisições idênticas disparadas repetidamente em sequência rápida (ex: temporizadores ou watchers em segundo plano).
* **Diagnóstico de Tráfego Recorrente:** Alerta integrado e modal didático agnóstico que explica causas de tráfego contínuo (polling de UI, tentativas repetidas em rotas 404 e reconexões agressivas de canais em tempo real).
* **Salvar Rota na Coleção:** Ícone de marcador (*bookmark*) exibido em cada card, permitindo adicionar qualquer requisição diretamente à Coleção do projeto com 1 clique.
* **Seleção Múltipla e Exclusão em Massa:** Modo de marcação para excluir requisições selecionadas ou limpar o histórico.

### `Inspector.svelte`
* **Foco Automático no Response:** O inspetor abre diretamente na aba **Response** por padrão ao selecionar qualquer requisição e ao concluir disparos manuais, agilizando a conferência de payloads de retorno.
* **Tabs Especializadas:** Alternância entre **Request**, **Response**, **Diff** (comparador lado a lado) e **cURL** (comando gerado pronto para o terminal).
* **Ações Rápidas de Cabeçalho:**
  * **Replay:** Abre a requisição com método, cabeçalhos e body pré-carregados para edição e reenvio.
  * **Salvar na Coleção:** Botão no topo para criar um template de rota reutilizável a partir da chamada inspecionada.
  * **Apagar:** Exclui a requisição atual do histórico.
* **Diagnóstico Contextual de Rotas e Polling:** Cards informativos contextuais quando uma rota retorna 404 (orientando sobre prefixos e métodos) ou quando é detectado polling rápido.

## 3. Gestão de Estado com Svelte 5 Runes
Toda a sincronização da interface é centralizada no `traffic.svelte.ts`, utilizando `$state` para as trocas HTTP, `$derived` para filtros e métricas em tempo real, e persistência em `localStorage` para estados de layout e preferências do desenvolvedor.
