# 백엔드 이미지. 1단계에서 릴리즈 빌드, 2단계는 바이너리만 담은 경량 런타임.
# 마이그레이션 SQL 은 sqlx::migrate! 로 바이너리에 내장되므로 복사하지 않는다.

# ---- 1단계: 빌드 ----
FROM rust:1.97-slim-bookworm AS builder
WORKDIR /app

# 의존성만 먼저 빌드해 캐시 (소스가 바뀌어도 의존성 재컴파일 방지)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY src ./src
COPY migrations ./migrations
# 더미 main.rs 로 만든 산출물이 재사용되지 않도록 갱신 시각을 바꾼다
RUN touch src/main.rs && cargo build --release --locked

# ---- 2단계: 런타임 ----
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 1000 destory

WORKDIR /app
COPY --from=builder /app/target/release/api /usr/local/bin/api
RUN mkdir -p /app/uploads && chown -R destory:destory /app

USER destory
ENV SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    UPLOAD_DIR=/app/uploads
EXPOSE 8080

HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/api/ping || exit 1

CMD ["api"]
