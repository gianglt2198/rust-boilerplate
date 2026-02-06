# Rust Hexagonal Boilerplate
A production-ready Rust web service template built with **Hexagonal Architecture (Ports and Adapters)**, utilizing a Rust Workspace to enforce strict dependency rules.

## 🏗 Architecture
This project follows strict Hexagonal Architecture principles to separate business logic from technology choices.

### The Dependency Rule
Dependencies flow inwards. The Core knows nothing about the Database or the Web Framework.

- **Core (Inner Layer)**: Pure business logic & interfaces (Ports). Zero external infrastructure dependencies.

- **Adapters (Middle Layer)**: Implementations of the Ports (Postgres/SeaORM, NATS, Redis). Depends on Core.

- **Apps (Outer Layer)**: The entry points (Web Server, Worker) that wire everything together. Depends on Core & Adapters.

## 📂 Project Structure
```Plaintext
.
├── apps/                    # 🚀 EXECUTION LAYER (Entry Points)
│   ├── api-server/          # Axum Web Server (HTTP Adapter)
│   └── worker/              # NATS Background Worker (Event Consumer)
│
├── crates/                  # 🧠 BUSINESS & LOGIC LAYER
│   ├── core/                # THE DOMAIN
│   │   ├── domain/entities  # Pure data structures (User, Order)
│   │   ├── domain/ports     # Interfaces (UserRepository, EventPublisher)
│   │   └── services/        # Business Logic (UserService)
│   │
│   └── adapters/            # 🔌 THE IMPLEMENTATION
│       ├── persistence/     # SeaORM/Postgres implementations
│       └── messaging/       # NATS implementations
│
└── libs/                    # 🛠 SHARED UTILITIES
    ├── common/              # Helper functions (IDs, etc.)
    ├── configuration/       # Typed Config Loader (Env/Yaml)
    └── telemetry/           # OpenTelemetry Setup (Tracing/Metrics)
```
    
## 🛠 Tech Stack
Language: Rust 2024 (Edition)

Web Framework: Axum

Database ORM: SeaORM (Postgres)

Messaging: NATS (Async NATS)

Observability: OpenTelemetry (Loki, Tempo, Prometheus, Grafana)

Runtime: Tokio

## 🚀 Getting Started
1. Prerequisites
Rust & Cargo

Docker & Docker Compose

2. Start Infrastructure
Start Postgres, NATS, and the Observability Stack:

```Bash
docker-compose up -d
```
3. Configuration
Copy the example environment file:

```Bash
cp .env.example .env
```
4. Run the Applications
You can run the API server and the Worker in separate terminals.

Start the API Server:

```Bash
cargo run -p api-server
```

Start the Background Worker:

```Bash
cargo run -p worker
```
## 👩‍💻 Development Workflow (How to add features)
When adding a new feature (e.g., "Create Order"), follow this flow from the inside out:

Core (Domain): Define the Order struct in `crates/core/domain/entities`.

Core (Port): Define the OrderRepository trait in `crates/core/domain/ports`.

Core (Service): Implement OrderService in `crates/core/services using the trait`.

Adapters: Implement the SeaOrmOrderRepository in `crates/adapters`.

App (API): Create a route in `apps/api-server` and call the Service.

App (Main): Wire the specific Adapter into the Service in `main.rs`.

## 📊 Observability
The stack includes full OTLP support.

Grafana: http://localhost:3000

Prometheus: http://localhost:9090

Tempo (Traces): http://localhost:3200

Jaeger/Zipkin: via Tempo

## 📜 License
MIT