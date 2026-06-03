# syntax=docker/dockerfile:1.7

FROM node:21

ARG GIT_SHA=unknown

WORKDIR /app

COPY package*.json ./

RUN --mount=type=cache,target=/root/.npm npm ci

COPY . .

EXPOSE 42069

ENV NODE_ENV=production
ENV PONDER_TELEMETRY_DISABLED=true
ENV GIT_SHA=${GIT_SHA}

LABEL org.opencontainers.image.revision=${GIT_SHA}

CMD [ "scripts/indexer.sh" ]

HEALTHCHECK --interval=10s --timeout=5s --start-period=60m --retries=10 \
  CMD node scripts/healthcheck.js
