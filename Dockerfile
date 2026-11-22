# ==============================================================================
# Colmeia — Container de Desenvolvimento e Validação (TDD Ready)
# Compatível com x86_64 e aarch64 (Apple Silicon / ARM)
# ==============================================================================
FROM rust:1.82-bookworm

# Evita interatividade durante a instalação de pacotes
ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH="/usr/local/cargo/bin:${PATH}"

# Instala dependências de sistema mínimas:
# - git >= 2.40 (obrigatório para o GitBackend do Colmeia)
# - sqlite3 e libsqlite3-dev (para rusqlite e inspeção direta)
# - build-essential, pkg-config, curl, procps (para /proc/meminfo e governor)
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    git \
    sqlite3 \
    libsqlite3-dev \
    curl \
    ca-certificates \
    procps \
    nano \
    tree \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Valida versão do Git instalada
RUN git --version

# Instala componentes de qualidade do Rust (clippy, rustfmt)
RUN rustup component add clippy rustfmt

# Instala ferramentas de desenvolvimento e TDD:
# - just: executor de tarefas determinístico (substituto moderno do Make)
# - cargo-watch: para ciclo TDD contínuo (red-green-refactor)
RUN cargo install just cargo-watch --locked

# Cria usuário não-root para desenvolvimento seguro
ARG USERNAME=developer
ARG USER_UID=1000
ARG USER_GID=1000

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m -s /bin/bash $USERNAME \
    && mkdir -p /workspace /usr/local/cargo/registry /usr/local/cargo/git \
    && chown -R $USERNAME:$USERNAME /workspace /usr/local/cargo

WORKDIR /workspace

# Configurações de ambiente de desenvolvimento
ENV RUST_BACKTRACE=1
ENV COLMEIA_TIER=t2

USER $USERNAME

# Porta padrão do servidor Web/SSE (read-only GUI)
EXPOSE 7420

CMD ["bash"]
