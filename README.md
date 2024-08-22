# BasePaint Indexer

This is the source code of the data backend used by https://basepaint.xyz/

## Getting Started

1. Clone the repo
2. Copy `.env.example` into `.env` and edit
3. `docker compose up`

## Deploying

Build and push new image

```
$ export GITHUB_SHA=$(git rev-parse HEAD)
$ export DOCKER_DEFAULT_PLATFORM=linux/amd64
$ docker login ghcr.io -u w1nt3r-eth
$ docker build -t ghcr.io/w1nt3r-eth/basepaint-ponder -t ghcr.io/w1nt3r-eth/basepaint-ponder:$GITHUB_SHA .
$ docker image push ghcr.io/w1nt3r-eth/basepaint-ponder:$GITHUB_SHA
$ docker image push ghcr.io/w1nt3r-eth/basepaint-ponder
```

Pull new image from the server

```
$ scp docker-compose.prod.yml ponder2.basepaint.xyz:docker-compose.yml
$ ssh ponder2.basepaint.xyz
$ docker login ghcr.io -u w1nt3r-eth
$ export GITHUB_SHA=...
$ docker compose up -d
$ docker system prune --all --force
```

