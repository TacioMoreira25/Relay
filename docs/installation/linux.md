# Guia de Instalação e Dependências para Linux

O **Relay** utiliza o motor nativo do **Tauri v2** com **WebKitGTK**. Para compilar ou rodar em modo de desenvolvimento em qualquer distribuição Linux, são necessários o compilador Rust, o ambiente Node.js e as bibliotecas nativas de desenvolvimento do GTK/WebKit.

---

## 1. Instalando Rust & Node.js (Todas as Distribuições)

### Compilador Rust (via Rustup)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### Node.js (v18+ ou v20+ / v22+)
Você pode usar o gerenciador de pacotes da sua distro ou o `fnm` / `nvm`:
```bash
# Exemplo com fnm (rápido e leve)
curl -fsSL https://fnm.vercel.app/install | bash
source ~/.bashrc
fnm install --lts
```

---

## 2. Dependências Nativas por Distribuição

### 🔴 Fedora / RHEL / AlmaLinux
```bash
sudo dnf install -y \
  gcc \
  gcc-c++ \
  webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel
```

### 🟠 Ubuntu / Debian / Pop!_OS / Linux Mint
```bash
sudo apt update && sudo apt install -y \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libwebkit2gtk-4.1-dev
```

### 🔵 Arch Linux / Manjaro / EndeavourOS
```bash
sudo pacman -Syu --needed \
  base-devel \
  curl \
  wget \
  openssl \
  gtk3 \
  libappindicator-gtk3 \
  librsvg \
  webkit2gtk-4.1
```

### 🟢 openSUSE (Tumbleweed / Leap)
```bash
sudo zypper install -y \
  gcc \
  gcc-c++ \
  gtk3-devel \
  webkit2gtk4.1-devel \
  libappindicator3-devel \
  librsvg-devel \
  openssl-devel
```

---

## 3. Inicialização Rápida

```bash
# 1. Clone o repositório
git clone https://github.com/seu-usuario/relay.git
cd relay

# 2. Instale as dependências da UI
npm install

# 3. Inicie em modo de desenvolvimento com Hot-Reload
npm run tauri dev

# 4. Ou gere o binário de produção (.rpm, .deb ou AppImage)
npm run tauri build
```
