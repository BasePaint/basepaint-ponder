FROM node:21

WORKDIR /app

COPY package*.json ./

RUN npm install

COPY . .

EXPOSE 42069

ENV NODE_ENV=production
ENV PONDER_TELEMETRY_DISABLED=true

CMD [ "npm", "start" ]
