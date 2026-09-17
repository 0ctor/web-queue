FROM node:22-alpine AS builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM nginx:1.27-alpine AS production
RUN apk add --no-cache curl
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html
LABEL org.opencontainers.image.source="https://github.com/0ctor/web-queue"
EXPOSE 80
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=5 \
  CMD curl -fsS http://127.0.0.1/ >/dev/null || exit 1
CMD ["nginx", "-g", "daemon off;"]
