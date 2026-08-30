# ADR 0001: Escolha da Stack Tecnológica (Tauri v2 + Rust + Svelte)

* **Status:** Aceito
* **Data:** 2026-08-29
* **Decisores:** Time de Arquitetura Relay

---

## 1. Contexto do Problema

Ferramentas modernas de desenvolvimento para interceptação e depuração de tráfego de rede (como Postman, Insomnia, Charles Proxy e Proxyman) tradicionalmente enfrentam graves desafios de desempenho no Linux:

1. **Alto Consumo de Memória (RAM):** Aplicações baseadas em Electron costumam consumir entre 300MB e 1GB de memória mesmo em repouso.
2. **Latência de Interceptação:** Motores de proxy implementados em NodeJS ou Python introduzem jitter e overhead perceptível ao processar fluxos pesados de dados (ex: downloads binários ou dezenas de chamadas concorrentes).
3. **Integração com o Desktop Linux:** Falta de integração nativa fluida com ambientes como KDE Plasma e Wayland, resultando em inicialização lenta e dependências de empacotamento pesadas.

---

## 2. Decisão

Decidimos construir o **Relay** utilizando:
* **Backend:** Rust (Tokio assíncrono + Hyper 1.x + Serde)
* **Shell Desktop:** Tauri v2 (utilizando o WebKitGTK nativo do Linux)
* **Frontend:** Svelte 5 + TypeScript + TailwindCSS

---

## 3. Justificativas e Análise Comparativa

### 3.1. Tauri v2 vs Electron
* **Tamanho do Binário:** Redução de ~120MB (Electron) para ~8MB-15MB (Tauri v2).
* **Consumo de Memória:** O footprint de RAM cai de ~400MB para ~25MB a 40MB.
* **Segurança e IPC:** O IPC do Tauri v2 utiliza comunicação binária serializada nativa com permissões granulares (`capabilities`), prevenindo injeções no contexto do sistema.

### 3.2. Rust (Hyper + Tokio) para o Motor de Proxy
* **Zero-Cost Abstractions & Zero-Copy:** O uso de buffers `bytes::Bytes` e streams assíncronos permite duplicar os pacotes de rede para visualização da UI sem cópia desnecessária na memória heap.
* **Concorrência Segura:** Sem garbage collector, garantindo latências de repasse de frações de milissegundo.
* **Resiliência:** O ecossistema Tokio/Hyper lida com milhares de conexões simultâneas com baixo uso de CPU.

### 3.3. Svelte para a Interface de Usuário
* **Sem Virtual DOM:** Svelte compila diretamente em código imperativo vanilla que atualiza cirurgicamente o DOM, ideal para renderizar dezenas de requisições por segundo na lista de tráfego sem congelamentos de frame (60/120 FPS no KDE Plasma).
* **Simplicidade de Manutenção:** Menos boilerplate reativo quando comparado com React/Redux.

---

## 4. Consequências e Trade-offs

### Pontos Positivos:
* Inicialização instantânea (< 200ms).
* Execução leve em segundo plano, perfeita para permanecer ativa no system tray.
* Estabilidade comprovada do compilador Rust para eliminar crashes em runtime.

### Pontos de Atenção / Desafios:
* Necessidade de empacotar bibliotecas nativas (`webkit2gtk4.1`) nas distribuições Linux alvo (Fedora/Debian/Arch).
* Curva de aprendizado inicial mais acentuada no Rust para manipulação de streams de requisições HTTP em baixo nível.
