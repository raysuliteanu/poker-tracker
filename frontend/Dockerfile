# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY package.json package-lock.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build the application with API URL (can be overridden at build time)
ARG VITE_API_URL=/api
ENV VITE_API_URL=${VITE_API_URL}

RUN npm run build

# Production stage
FROM nginx:alpine

# Install wget for healthcheck and gettext for envsubst
RUN apk add --no-cache wget gettext

# Copy built assets from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx template instead of conf
COPY nginx.conf.template /etc/nginx/templates/default.conf.template

# Set default environment variables for local development
ENV BACKEND_URL=http://backend:8080
ENV BACKEND_HOST=backend

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
