FROM rustlang/rust:nightly-bookworm AS chef
RUN rustup component add --toolchain nightly rust-docs-json
RUN cargo install cargo-chef cargo-px
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM node:21-slim as frontend
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY frontend /app/frontend
WORKDIR /app/frontend

RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

FROM chef AS builder
ENV APP_ENV=production
RUN cargo install --locked --git "https://github.com/LukeMathWalker/pavex.git" --branch "main" pavex_cli
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
COPY --from=frontend /app/frontend/dist /app/frontend/dist
RUN cargo px build --release --bin server

FROM debian:bookworm-slim as runtime
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY config /app/config
COPY --from=builder /app/target/release/server /usr/local/bin

# Required to properly load default config from files
RUN mkdir /app/server

ENV APP_ENV=production

EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/server"]

HEALTHCHECK --interval=5m \
	CMD curl -f http://localhost:8000/healthz || exit 1
