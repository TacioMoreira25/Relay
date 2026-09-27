#!/usr/bin/env bash
set -e

echo "[Relay] Iniciando verificacao automatizada do ambiente..."

# 1. Checagem do Node.js
if ! command -v node >/dev/null 2>&1; then
    echo "[ERRO] Node.js nao encontrado. Instale o Node.js v20+ (LTS recomendado)."
    exit 1
fi

NODE_MAJOR=$(node -v | cut -d'.' -f1 | tr -d 'v')
if [ "$NODE_MAJOR" -lt 20 ]; then
    echo "[AVISO] Versao do Node.js detectada ($(node -v)). Recomendado Node.js >= 20.0.0."
fi

# 2. Checagem do Rust / Cargo
if ! command -v cargo >/dev/null 2>&1; then
    echo "[ERRO] Cargo/Rust nao encontrado."
    echo "Instale executando: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 3. Informacao de pacotes de sistema para Linux
if [ "$(uname -s)" = "Linux" ]; then
    echo "[Relay] Verificando requisitos de sistema Linux (Tauri v2)..."
    if command -v apt-get >/dev/null 2>&1; then
        if ! dpkg -s libwebkit2gtk-4.1-dev build-essential libssl-dev >/dev/null 2>&1; then
            echo "[DICA] Para distribuiçoes Debian/Ubuntu/Mint, certifique-se de instalar as dependencias:"
            echo "sudo apt update && sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev"
        fi
    fi
fi

# 4. Instalacao deterministica das dependencias do Frontend
echo "[Relay] Instalando dependencias do frontend com lockfile estrito (npm ci)..."
npm ci

# 5. Verificacao do Backend
echo "[Relay] Verificando compilacao do backend Rust..."
cargo check --manifest-path src-tauri/Cargo.toml

echo ""
echo "[Relay] Ambiente de desenvolvimento configurado com sucesso!"

# 6. Deteccao automatica de instalacao existente no sistema operacional
IS_INSTALLED=false

if command -v relay >/dev/null 2>&1; then
    IS_INSTALLED=true
elif command -v rpm >/dev/null 2>&1 && rpm -qa | grep -qi "^relay"; then
    IS_INSTALLED=true
elif command -v dpkg >/dev/null 2>&1 && dpkg -s relay >/dev/null 2>&1; then
    IS_INSTALLED=true
fi

# Se for solicitada instalacao (--install / --update) OU se o app ja estiver instalado no sistema operacional (e nao for --dev-only)
if [ "$1" = "--install" ] || [ "$1" = "--update" ] || [ "$IS_INSTALLED" = true -a "$1" != "--dev-only" ]; then
    if [ "$IS_INSTALLED" = true ]; then
        echo ""
        echo "[Relay] Aplicativo instalado detectado no sistema operacional. Atualizando versao..."
    else
        echo ""
        echo "[Relay] Instalando aplicativo desktop no sistema operacional..."
    fi

    if command -v dnf >/dev/null 2>&1; then
        echo "[Relay] Compilando pacote RPM..."
        npm run tauri build -- --bundles rpm
        RPM_FILE=$(find src-tauri/target/release/bundle/rpm -name "*.rpm" 2>/dev/null | head -n 1)
        if [ -n "$RPM_FILE" ]; then
            echo "[Relay] Atualizando aplicativo via dnf..."
            sudo dnf reinstall -y "$RPM_FILE" || sudo dnf install -y "$RPM_FILE"
            echo ""
            echo "[Relay] Aplicativo atualizado no sistema operacional com sucesso!"
        fi
    elif command -v apt-get >/dev/null 2>&1; then
        echo "[Relay] Compilando pacote DEB..."
        npm run tauri build -- --bundles deb
        DEB_FILE=$(find src-tauri/target/release/bundle/deb -name "*.deb" 2>/dev/null | head -n 1)
        if [ -n "$DEB_FILE" ]; then
            echo "[Relay] Atualizando aplicativo via dpkg..."
            sudo dpkg -i "$DEB_FILE"
            echo ""
            echo "[Relay] Aplicativo atualizado no sistema operacional com sucesso!"
        fi
    else
        echo "[Relay] Compilando binario portatil..."
        npm run tauri build
        mkdir -p "$HOME/.local/bin"
        cp src-tauri/target/release/relay "$HOME/.local/bin/"
        echo ""
        echo "[Relay] Binario atualizado em $HOME/.local/bin/relay com sucesso!"
    fi
else
    echo ""
    echo "Dicas:"
    echo "  npm run tauri dev   -> Inicia em modo de desenvolvimento"
    echo "  ./setup.sh --install -> Compila e instala o aplicativo no sistema operacional"
fi
