# Rust GraphQL Template

**THIS PROJECT IS STILL ONGOING, USE IT ON YOUR OWN RISK**

A full feature template to develop server-side GraphQL APIs using Rust for data and processor heavy APIs.

<small>**Note:** this project is not endorsed by the Rust Foundation</small>

## Technologies used

For this template we use the following frameworks and libraries:

- **[Actix-Web](https://actix.rs/):** the most common used back-end framework in the Rust ecosystem;
- **[Async-GraphQL](https://async-graphql.github.io/async-graphql/en/index.html):** GraphQL adaptor;
- **[SeaOrm](https://www.sea-ql.org/SeaORM/)**: a SQL ORM for interacting with the database;

## What is implemented

Three services are implemented:

- Local and External OAuth 2.0 authentication in REST;
- User CRUD;
- Image upload with compression (it needs to be improved, very slow performance);

## How to use it

Just click on the `Use this template` button and follow the instructions.

## Project Structure

The project is divided into workspaces:

1. **Entities:** here is where we define our DB data models;
2. **Migrations:** here is where we define our DB migrations, I left it as a code first, so it uses a `Schema` to
   auto-generate the table migration structure;
3. **App:** here is where we define our business logic, it is divided into modules, each module has its own
   responsibility;

The App tries to follow a basic MSC/MSR (Model, Service, Controller/Resolver) pattern, where the:

- `Model` is the DTO (Data Transfer Object) GQL Object that represents the data that will be shown by the API;
- `Service` is the business logic that will be used to process the data for a specific Data Model;
- `Controller`: only applies to AUTH, it is the REST controller that will be used to authenticate the user;
- `Resolver` is the GraphQL resolver that will be used to expose in the API the data for a specific Data Model;
- `Providers` are the external services that will be used by the App, like the DB, the OAuth2 provider, etc;

## How to run it locally

### OS and Software requirements

This template requires a MacOS or Linux machine with the following applications installed:

- _[Docker](https://www.docker.com/) or [Podman](https://podman.io/)_: to be able to run the PostgreSQL database, Redis Cache and the Object Storage;
- _[Rust](https://www.rust-lang.org/)_: to be able to run and devolep this template;
- _[AWS CLI](https://aws.amazon.com/pt/cli/)_: to be able to run the Object Storage;

### Configuration

Create a `.env` file in the root of the project with the following content:

```dotenv
# Environment Setup
ENV="development"

# TCP port
PORT=5000

# DBs Setup
REDIS_URL="redis://localhost:6379"
DATABASE_URL="postgresql://postgres:postgres@localhost:5432/somedb"

# Jwt OAuth Setup
ACCESS_SECRET="random_string"
ACCESS_TIME=600
CONFIRMATION_SECRET="random_string"
CONFIRMATION_TIME=3600
RESET_SECRET="random_string"
RESET_TIME=1800
REFRESH_SECRET="random_string"
REFRESH_TIME=604800
REFRESH_NAME="cookie_name"

# Email Setup
EMAIL_HOST="smtp.gmail.com"
EMAIL_PORT=587
EMAIL_USER="johndoe@gmail.com"
EMAIL_PASSWORD="your_email_password"

# URL Setup
API_ID="00000000-0000-0000-0000-000000000000"
FRONTEND_URL="http://localhost:3000"
BACKEND_URL="http://localhost:5000"

# External OAuth Setup
GOOGLE_CLIENT_ID="000000000000"
GOOGLE_CLIENT_SECRET="000000000000"
FACEBOOK_CLIENT_ID="000000000000"
FACEBOOK_CLIENT_SECRET="000000000000"

# Object Storage Setup
OBJECT_STORAGE_BUCKET="test"
OBJECT_STORAGE_SECRET_KEY="test"
OBJECT_STORAGE_ACCESS_KEY="test"
OBJECT_STORAGE_REGION="us-east-1"
OBJECT_STORAGE_HOST="localhost:4566"
OBJECT_STORAGE_NAMESPACE="00000000-0000-0000-0000-000000000000"
```

## How to run it

Install the CLI tools for Database creation (sqlx-cli) and migrations (sea-orm-cli):

```bash
cargo install sqlx-cli
cargo install sea-orm-cli
```

Run docker and set up the object storage:

```bash
docker-compose up -d
aws configure --profile localstack
# AWS Access Key ID: test
# AWS Secret Access Key: test
# Default region name: us-east-1
# Default output format: json
aws --endpoint-url=http://localhost:4566 --profile localstack s3 mb s3://test
```

Run the migrations:

```bash
sqlx database create
sea-orm-cli migrate -d migrations
```

After this you have two options to run the project:

- Locally;
- In a docker container.

### Run locally

Run the project:

```bash
cargo run
```

### Run in docker container

Build the docker image and run the project:

```bash
docker build -t graphqlapi .
docker-compose -f compose.yaml -f compose.apps.yaml up  
```

## How to test it

This project only has E2E tests, so you need to have the DB running to be able to run the tests. On the resolvers and on the controllers
you can find the `tests.rs` file, there you can find the tests for each resolver and controller.

1. Change the DB on `.env` to something like `DATABASE_URL="postgresql://postgres:postgres@localhost:5432/somedb_test"`;
2. Start the DB:
   ```bash
   sqlx database create
   sea-orm-cli migrate -d migrations
   cargo test
   ```
3. To run the tests:
   ```bash
   cargo test
   ```

## How to deploy it

TODO: create Dockerfile for deployment

<small>Note to self: deploy to Dokku? Or kubernetes? Or both?</small>

## License

This template is licensed under the Mozilla Public License 2.0, see the [LICENSE](LICENSE) file for details.
