# my-ex-review-service

Review / Analytics microservice for Experience Review platform. **Rust + Axum.**

## Stack

- Rust 1.83
- Axum 0.7
- SQLx + PostgreSQL
- Tower (CORS)

## Endpoints

- `GET /health` — health check
- `GET /reviews` — list reviews
- `GET /reviews/:id` — get review
- `POST /reviews` — create review (body: `product_id`, `user_id`, `rating`, `body`)
- `GET /stats/dashboard` — dashboard stats (`total_reviews`, `avg_rating`)

## Run locally

```bash
# Install Rust: https://rustup.rs
cargo build
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/review_db
# Run migrations: sqlx migrate run (or run service; migrations run on startup)
cargo run
```

## Docker Compose

```bash
docker compose up --build
# Service: http://localhost:3005
```

## Kubernetes

```bash
kubectl apply -f k8s/
# DB first: kubectl apply -f k8s/db-deployment.yaml
# Then: kubectl apply -f k8s/deployment.yaml k8s/service.yaml
# Update secret database-url to match review-db service if needed
```

## Env vars

| Variable      | Default | Description        |
|---------------|---------|--------------------|
| PORT          | 3005    | Server port        |
| DATABASE_URL  | -       | Postgres URL       |

## Cargo

No compile-time DB required; SQLx is used in runtime mode. Commit `Cargo.lock` for reproducible builds.
