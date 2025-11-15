# Multi-stage build for Sentiment Intelligence Platform

# Stage 1: Build TypeScript
FROM node:18-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY tsconfig.json ./

# Install dependencies
RUN npm ci --only=production && \
    npm install --save-dev typescript @types/three

# Copy source
COPY app.ts ./
COPY index.html ./

# Build TypeScript
RUN npm run build

# Stage 2: Production image with Nginx
FROM nginx:alpine

# Copy built files
COPY --from=builder /app/index.html /usr/share/nginx/html/
COPY --from=builder /app/dist /usr/share/nginx/html/dist/

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://localhost:80/ || exit 1

# Expose port
EXPOSE 80

# Labels
LABEL maintainer="Cody Moore <cody.moore@outlook.com>"
LABEL description="Sentiment Intelligence Platform - Production SaaS Application"
LABEL version="1.0.0"

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
