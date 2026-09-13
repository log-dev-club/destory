# 프론트엔드 이미지. 1단계에서 Vite 빌드, 2단계는 Caddy 가 정적 파일 서빙 + /api 프록시 + HTTPS.

# ---- 1단계: 빌드 ----
FROM node:24-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

# ---- 2단계: Caddy ----
FROM caddy:2-alpine
COPY Caddyfile /etc/caddy/Caddyfile
COPY --from=builder /app/dist /srv
EXPOSE 80 443
