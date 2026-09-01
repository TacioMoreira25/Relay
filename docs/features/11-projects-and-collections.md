# Gerenciamento de Projetos, Importação de Coleções e Pastas Dinâmicas

## 1. Multi-Projetos (Workspaces Isolados)

O Relay organiza o fluxo de desenvolvimento em **Workspaces por Projeto**. Cada projeto possui seu próprio isolamento de:
* Configurações de Portas e Hosts
* Rotas de Mapeamento e Mocks
* Coleções de Rotas e Templates
* Ambientes Registrados

### Alternando e Gerenciando Projetos:
No menu **Projetos** (canto superior esquerdo da TopBar):
* **Criar Projeto:** Permite criar novos espaços de trabalho para APIs diferentes (ex: "Sistema de Vendas", "Microsserviço Auth").
* **Alternar Projeto:** Salva automaticamente o estado atual e carrega a configuração e coleções do projeto selecionado sem reiniciar o app.
* **Persistência Total:** Todas as configurações e coleções são salvas em tempo real no `localStorage` do cliente e sincronizadas no backend Rust.

---

## 2. Importador Inteligente e Universal de Coleções

O botão **Importar JSON** no topo da barra lateral suporta automaticamente três padrões da indústria sem necessidade de conversão manual:

### Formatos Suportados:
1. **OpenAPI 3.0.x / Swagger 2.0 (JSON):**
   * Lê as definições de `paths` e métodos HTTP (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`).
   * Extrai resumos, descrições e identificadores de tags.
   * Constrói exemplos automáticos de payload JSON baseados no `requestBody.content.application/json.schema`.
   * Identifica requisitos de autenticação `security`.
2. **Postman Collection v2.0 / v2.1:**
   * Lê pastas aninhadas, nomes de endpoints, URLs com variáveis, cabeçalhos e corpos raw.
3. **JSON Array Nativo Relay:**
   * Aceita estruturas flexíveis com campos `method`, `uri`/`url`/`path`/`endpoint`, `headers`, `body` e `tag`.

---

## 3. Pastas Retráteis Dinâmicas (Accordion)

Na aba **Coleção**, as rotas importadas são organizadas automaticamente em **Pastas Retráteis (Accordion)**:
* **Detecção Automática:** As pastas são geradas dinamicamente com base nas `tags` do OpenAPI/Postman ou pelo primeiro segmento da URL (`/atendimentos/...` -> Pasta `atendimentos`).
* **Contadores de Rotas:** Cada pasta exibe a quantidade total de endpoints vinculados.
* **Expansão com 1 Clique:** As pastas podem ser abertas ou recolhidas individualmente, mantendo a tela limpa mesmo com dezenas de rotas cadastradas.
