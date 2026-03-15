# syntax=docker/dockerfile:1
FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /build

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY resources ./resources
RUN cargo build --release --bin research

FROM debian:trixie-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    python3 \
    python3-venv \
    python3-dev \
    libglib2.0-0 \
    libpango-1.0-0 \
    libpangoft2-1.0-0 \
    libpangocairo-1.0-0 \
    libcairo2 \
    libgdk-pixbuf-2.0-0 \
    libffi8 \
    libssl3t64 \
    fontconfig \
    fonts-noto-cjk \
    fonts-noto-color-emoji \
    fonts-dejavu \
    && rm -rf /var/lib/apt/lists/*
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:$PATH"
RUN uv run --with weasyprint python3 -c "import weasyprint"
COPY pyproject.toml ./pyproject.toml
RUN uv sync
COPY resources ./resources
COPY --from=builder /build/target/release/research /usr/local/bin/research
ENV RESOURCES_DIR=/app/resources
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8
ENTRYPOINT ["research"]
