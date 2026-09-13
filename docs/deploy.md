# 배포 (Docker Compose)

PostgreSQL, 백엔드(axum), 프론트+프록시(Caddy) 세 컨테이너를 `docker-compose.yml` 하나로 띄운다.
Docker 만 설치되어 있으면 클라우드 VM 과 개인 PC 에서 같은 명령으로 동작한다.

```
브라우저 ─▶ web (Caddy, 80/443) ─┬─ /api/* ─▶ api (axum, 내부 8080) ─▶ db (PostgreSQL, 내부 5432)
                                 └─ 그 외  ─▶ 정적 파일 (Vite 빌드 결과)
```

## 1. 준비물

- Docker Engine + Compose v2 (`docker compose version` 으로 확인)
  - 리눅스 서버: https://docs.docker.com/engine/install/
  - Windows/Mac 개인 PC: Docker Desktop
- 저장소 clone
- (도메인으로 HTTPS 를 쓸 경우) 도메인의 A 레코드가 서버 IP 를 가리키고, 80·443 포트가 외부에 열려 있어야 함

## 2. 환경 변수

```bash
cp .env.example .env
```

| 키 | 필수 | 설명 |
|---|---|---|
| `POSTGRES_PASSWORD` | O | DB 비밀번호. 컨테이너 내부 전용이지만 강한 값 |
| `SITE_ADDRESS` | O | Caddy 바인딩. `:80` (HTTP) 또는 `destory.example.com` (HTTPS 자동) |
| `APP_BASE_URL` | O | 브라우저 접속 주소. Discord 링크·GitHub 콜백 기준 |
| `COOKIE_SECURE` | O | HTTPS 면 `true`, HTTP 면 `false`. 틀리면 로그인이 안 됨 |
| `GITHUB_CLIENT_ID` / `SECRET` | – | 배포 주소용 OAuth App. 비우면 GitHub 로그인 비활성 |
| `DISCORD_WEBHOOK_URL` | – | 비우면 알림 비활성 |

`api/.env` 는 로컬 개발(`cargo run`)용이고 Docker 는 읽지 않는다. 루트 `.env` 만 사용한다.

### 환경별 예시

**클라우드 VM + 도메인 (HTTPS)**
```
SITE_ADDRESS=destory.example.com
APP_BASE_URL=https://destory.example.com
COOKIE_SECURE=true
```
Caddy 가 Let's Encrypt 인증서를 자동 발급·갱신한다. 별도 설정 없음.

**개인 PC, 내부망 (HTTP)**
```
SITE_ADDRESS=:80
APP_BASE_URL=http://192.168.0.10      # 그 PC 의 내부 IP
COOKIE_SECURE=false
```
같은 네트워크의 팀원이 `http://192.168.0.10` 으로 접속. 외부에서 접속하려면 공유기 포트포워딩(80) 과 DDNS 가 필요하다.

**본인 PC 에서 테스트**
```
SITE_ADDRESS=:80
APP_BASE_URL=http://localhost
COOKIE_SECURE=false
```

## 3. 실행

```bash
docker compose up -d --build     # 이미지 빌드 + 백그라운드 실행
docker compose ps                # 세 컨테이너 모두 healthy/running 인지
docker compose logs -f api       # 백엔드 로그 (DB 생성, 마이그레이션, 기동 확인)
```

첫 빌드는 Rust 컴파일 때문에 5~10분 걸린다. 이후에는 의존성 캐시로 빨라진다.
백엔드가 시작하면서 DB 생성과 마이그레이션을 자동으로 처리한다.

## 4. 업데이트

```bash
git pull origin main
docker compose up -d --build     # 바뀐 이미지만 다시 빌드하고 교체
```

## 5. 데이터 위치와 백업

데이터는 Docker 볼륨에 있어 컨테이너를 지워도 남는다.

| 볼륨 | 내용 |
|---|---|
| `db-data` | PostgreSQL 데이터 |
| `uploads` | 첨부파일 |
| `caddy-data` | HTTPS 인증서 |

백업:
```bash
docker compose exec db pg_dump -U destory destory > backup.sql
docker run --rm -v destory_uploads:/data -v "$PWD":/backup alpine tar czf /backup/uploads.tgz -C /data .
```

복원(새 서버로 이전 시):
```bash
docker compose up -d db
docker compose exec -T db psql -U destory destory < backup.sql
docker run --rm -v destory_uploads:/data -v "$PWD":/backup alpine tar xzf /backup/uploads.tgz -C /data
docker compose up -d
```

## 6. 자주 겪는 문제

| 증상 | 원인 / 해결 |
|---|---|
| 로그인 후 바로 풀림 | `COOKIE_SECURE=true` 인데 HTTP 로 접속. HTTP 면 false |
| GitHub 로그인 시 redirect_uri 오류 | OAuth App 콜백 URL 이 `{APP_BASE_URL}/api/auth/github/callback` 과 다름 |
| 첨부 업로드 413 | `MAX_UPLOAD_MB` 조정 (Caddy 와 api 양쪽에 같은 값이 전달됨) |
| HTTPS 인증서 발급 실패 | 도메인 DNS 가 서버를 안 가리키거나 80/443 이 막힘. `docker compose logs web` |
| `api` 가 계속 재시작 | `docker compose logs api` 에서 panic 메시지 확인. 대부분 `.env` 값 문제 |

## 7. 중지 / 삭제

```bash
docker compose down              # 컨테이너만 중지·삭제 (데이터 유지)
docker compose down -v           # 볼륨까지 삭제 (DB·첨부파일 전부 사라짐, 주의)
```
