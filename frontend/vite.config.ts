import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    // 개발 서버에서 /api 요청을 axum 백엔드로 프록시
    // 팀원 참고: 백엔드 포트를 바꿨다면 (api/.env 의 SERVER_PORT) 여기도 맞춰주세요.
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
})
