FROM node:lts

RUN npm install concurrently -g

WORKDIR /orbix-web

RUN git clone https://github.com/orbix/orbix-web --depth 1 .

RUN npm install

EXPOSE 9050
EXPOSE 5252
EXPOSE 9060

CMD concurrently "npm run re:build && npm run start" "npm run start:playground"