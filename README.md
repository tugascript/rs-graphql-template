# Rust GraphQL Template

## **THIS PROJECT IS STILL ON GOING AND UNTESTED, USE IT AT YOUR OWN RISK**

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
- File upload;

## How to use it

Just click on the `Use this template` button and follow the instructions.

### Project Structure

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

### How to run it

To run the project you need to have the following tools installed:

- Docker or Podman: to be able to run the DB and Cache;
- Rust: to be able to run the App;

Create a `.env` file in the root of the project with the following content:

```dotenv
# DBs Setup
REDIS_URL="redis://redis:6379"
DATABASE_URL="postgres://postgres:postgres@db:5432/myapp"

# Jwt OAuth Setup
ACCESS_SECRET="random_string"
ACCESS_TIME=600
CONFIRMATION_SECRET="random_string"
CONFIRMATION_TIME=3600
RESET_PASSWORD_SECRET="random_string"
RESET_PASSWORD_TIME=1800
REFRESH_SECRET="random_string"
REFRESH_TIME=604800
REFRESH_NAME="cookie_name"
API_ID="00000000-0000-0000-0000-000000000000"

# Email Setup
EMAIL_HOST="smtp.gmail.com"
EMAIL_PORT=587
EMAIL_USER="johndoe@gmail.com"
EMAIL_PASSWORD="your_email_password"
FRONT_END_URL="http://localhost:3000"

# External OAuth Setup
GOOGLE_CLIENT_ID="000000000000"
GOOGLE_CLIENT_SECRET="000000000000"
FACEBOOK_CLIENT_ID="000000000000"
FACEBOOK_CLIENT_SECRET="000000000000"

# Object Storage Setup
BUCKET_NAME="linode_or_aws_bucket_name"
BUCKET_SECRET_KEY="bucket_secret_key"
BUCKET_ACCESS_KEY="bucket_access_key"
BUCKET_REGION="bucket_region"
BUCKET_HOST="aws?linode?digitalocean?"
USER_NAMESPACE="00000000-0000-0000-0000-000000000000"
```

Run the following commands.

```bash
$ docker-compose up -d
$ cargo run
```

## How to deploy it

TODO: create Dockerfile for deployment

<small>Note to self: deploy to Dokku? Or kubernetes? Or both?</small>

## License

This template is licensed under the Mozilla Public License 2.0, see the [LICENSE](LICENSE) file for details.
