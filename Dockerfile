FROM node:21

WORKDIR /app

COPY package*.json ./

RUN npm install

COPY . .

EXPOSE 42069

ENV NODE_ENV=production
ENV PONDER_TELEMETRY_DISABLED=true
ENV NODE_OPTIONS=--max_old_space_size=6096

CMD [ "npm", "start" ]
