# Rust GraphQL Template

## Overview

This simple template is designed to simplify the development of server-side GraphQL APIs using Rust, by providing a
basic structure, boilerplate code and some common functionalities.

This template is particularly suited for data-intensive and processor-heavy APIs.

**Disclaimer:** This project is not endorsed by the Rust Foundation.

## Technologies

This template incorporates several leading technologies and frameworks:

- **[Actix-Web](https://actix.rs/):** the most common used back-end framework in the Rust ecosystem;
- **[Async-GraphQL](https://async-graphql.github.io/async-graphql/en/index.html):** GraphQL adaptor;
- **[SeaOrm](https://www.sea-ql.org/SeaORM/)**: a SQL ORM for interacting with the database;

## Features

Currently, the following features are implemented:

### Error Handling

- Custom error handling with `Into<T>` and `From<T>` traits, to be compatible with both GraphQL and REST APIs default error handling.

### Authentication

- [JWT](https://jwt.io/) [OAuth2](https://oauth.net/2/) authentication with refresh token;
- [Facebook](https://facebook.com/) and [Google](https://google.com) OAuth2 authentication;
- Two-factor authentication with email.

### Basic CRUD operations

- User CRUD opeations in GraphQL

### File Upload

- Generic S3 compatible Object Storage upload with [Rusoto S3](https://crates.io/crates/rusoto_s3);
- Image upload with compression using the [Image crate](https://crates.io/crates/image) (Performnance improvements may be required for heavy loads).

## Usage Instructions

To utilize this template:

1. Click `Use this template` button on the top of the repository;
2. Choose a name for your project;
3. Mofify the following files to fit your project:
   - `README.md` to reflect your project's specifics;
   - `Cargo.toml` for the project's name and version.
4. Update the file headers with your copyright information, adhering to the MPLv2.0 license.

## Project Structure

### Workspaces

The project is organized into the following workspaces:

1. **Entities:** defines the database data models;
2. **Migrations:** manages the database migrations using a code-first approache
with `Schema`;
3. **App:** the core business logic, divided into distinct modules following MSC pattern.

### App Structure

The application adheres closely to a MSC/MSR (Model, Service, Controller/Resolver) pattern:

- `Model`: the DTOs (Data Transfer Object) and GQL Objects representing API-exposed data;
- `Service`: processes data for specific Data Models, and is responsible for the business logic;
- `Controller`: manages REST-based API endpoints, mostly responsible only for user authentication;
- `Resolver`: handles GraphQL queries and mutations for specific Data Models;
- `Providers`: external services used by the app (e.g., PostgreSQL, Redis).

## System Requirements

Apart from [Rust](https://www.rust-lang.org/), to develop and run this template, ensure your system meets the following prerequisites:

- _Operating System_: MacOS or Linux (it is untested on Windows);
- _[Docker](https://www.docker.com/) or [Podman](https://podman.io/)_: required for running the PostgreSQL database, Redis Cache, and Object Storage, and for deployment.
- _[AWS CLI](https://aws.amazon.com/cli/)_: to be able to run the Object Storage;

## Configuration

Create a `.env` file in the root of the project with the following content:

```dotenv
# Environment Setup
ENV="development"

# TCP port and host
PORT=5000
HOST="127.0.0.1"

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

## Running the project

### Initial Setup

1. Install CLI tools for database setup and migrations:
   ```bash
   cargo install sqlx-cli
   cargo install sea-orm-cli
   ```
2. Initialize Docker:
   ```bash
   docker-compose up -d
   ```
3. Set up object storage:
   ```bash
   aws configure --profile localstack
   # AWS Access Key ID: test
   # AWS Secret Access Key: test
   # Default region name: us-east-1
   # Default output format: json
   aws --endpoint-url=http://localhost:4566 --profile localstack s3 mb s3://test
   ```
4. Create the Database and run the migrations:
   ```bash
   sqlx database create
   sea-orm-cli migrate -d migrations
   ```

### Running Options

- Locally:
   ```bash
   cargo run
   ```
- Within a Docker container:
   ```bash
   docker build -t graphqlapi .
   docker-compose -f compose.yaml -f compose.apps.yaml up
   ```

## Testing

The project only includes end-to-end (E2E) tests:
1. Set the `.env` file with a test database;
2. Create the test Database and run the migrations:
   ```bash
   sqlx database create
   sea-orm-cli migrate -d migrations
   ```
3. To run the tests:
   ```bash
   cargo test
   ```

## License

This template is licensed under the Mozilla Public License 2.0, see the [LICENSE](LICENSE) file for details.
