# 🔍 Funcionalidade: Inspecionador de Tráfego em Tempo Real

## 1. Visão Geral
O Inspecionador exibe de forma reativa e sem latência perceptível todas as requisições que transitam pelo proxy, divididas em uma visualização mestre-detalhe (**Request List** e **Inspector Panel**).

## 2. Componentes e Responsabilidades

### `RequestList.svelte`
* **Lista Ordenada Cronologicamente:** As requisições mais recentes aparecem no topo com badges de método (`GET`, `POST`, `PUT`, `DELETE`), URI truncada, status code e tempo de execução (`ms`).
* **Status Visual Dinâmico:**
  * Status 2xx: Verde (`text-emerald-400`)
  * Status 3xx: Ciano (`text-cyan-400`)
  * Status 4xx: Âmbar (`text-amber-400`)
  * Status 5xx / Erro: Vermelho (`text-rose-400`)
  * Pendente: Âmbar pulsante

### `Inspector.svelte`
* **Tabs Separadas:** Alternância instantânea entre **Request** e **Response**.
* **Tabela de Headers Estruturada:** Mapeamento claro de chaves e valores enviados e retornados.
* **Formatador de JSON:** Exibição do payload body com seleção rápida de texto e suporte a caracteres especiais.

## 3. Gestão de Estado com Svelte 5 Runes
Toda a sincronização da interface é centralizada no [`traffic.svelte.ts`], utilizando `$state` para as trocas HTTP e `$derived` para contadores derivados em tempo real sem renderizações desnecessárias.
