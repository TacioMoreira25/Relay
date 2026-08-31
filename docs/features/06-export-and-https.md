# Exportação de Tráfego & Suporte a HTTPS (Root CA)

O Relay inclui ferramentas nativas para exportação padronizada de sessões de rede e geração de certificados raiz para inspeção SSL/TLS.

---

## 1. Exportação HAR 1.2 (HTTP Archive)

Permite salvar todo o tráfego capturado no formato oficial HAR 1.2:
- Compatível com Chrome DevTools, Postman, Insomnia, Charles Proxy e ferramentas de QA.
- Inclui métodos, URIs, cabeçalhos de request/response, corpos codificados, status codes e timings de latência em milissegundos.

## 2. Geração Automática de OpenAPI 3.0 (Swagger)

Converte automaticamente os endpoints interceptados durante a sessão em uma especificação viva OpenAPI 3.0.3:
- Agrupamento automático de rotas e métodos HTTP (`GET`, `POST`, `PUT`, `DELETE`, etc.).
- Geração de schemas de resposta com base nos payloads observados em tempo de execução.

## 3. Emissão Dinâmica de Certificados Root CA

Para permitir a inspeção transparente de conexões HTTPS sem erros de certificado:
- Emissão em memória de chaves criptográficas RSA/ECDSA e certificados X.509 padrão (`rcgen`).
- Download direto dos arquivos `relay-root-ca.crt` e `relay-root-ca.key`.
