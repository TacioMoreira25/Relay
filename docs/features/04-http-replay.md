# 🔁 Funcionalidade: Cliente HTTP & Replay de Chamadas

## 1. Visão Geral
Permite repetir qualquer requisição previamente capturada pelo Relay com apenas 1 clique, com possibilidade de modificar headers, query params ou o JSON body antes do reenvio.

## 2. Casos de Uso
* Reproduzir cenários de bug identificados em logs locais.
* Testar validações de formulário alterando parâmetros individuais.
* Auto-injetar o token JWT mais recente capturado pela sessão sem precisar atualizar headers manualmente.

## 3. Componentes
* `HeaderEditor.svelte`: Permite adicionar, editar e remover headers dinamicamente com inputs vinculados reativamente via `$bindable()`.
