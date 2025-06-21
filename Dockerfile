FROM node:21

WORKDIR /app

COPY package*.json ./

RUN npm install

COPY . .

EXPOSE 42069

ENV NODE_ENV=production
ENV PONDER_TELEMETRY_DISABLED=true

CMD [ "scripts/indexer.sh" ]

HEALTHCHECK --interval=10s --timeout=5s --start-period=60m --retries=10 \
  CMD node scripts/healthcheck.js
