# Less is More
A one-shot GPU SaaS platform

## Requirements
- Bazel
- Rust
- NextJS
- Docker
- Goose

## Running
Start the backend services in docker-compose

```
docker-compose up -d
```

Run migrations (only needs to be done once)

```
goose up
```

Start the backend rust server
```
FLY_API_TOKEN='<your token here>' bazel run --@rules_rust//rust/toolchain/channel=nightly :lim
```

Start the frontend server
```
npm run dev
```

## Todo
- Add a CPU type selector
- Add a way to view logs
- Add a way to view machine status
