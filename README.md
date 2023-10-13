# Rust GraphQL Template

A full feature template to develop Monolithic GraphQL APIs using Rust for data and processor heavy APIs.

## Technologies used

For this template we use the following frameworks and libraries:

- [Actix-Web]: the most common used back-end framework in the Rust ecosystem;
- [Async-GraphQL]: GraphQL adaptor;
- [SeaOrm]: a SQL ORM for interacting with the database;

**NOTE**: THIS TEMPLATE IS ON GOING, AND FOR SMALL PROJECTS ONLY. PLEASE ADD CACHING IF YOU WANT TO USE IT IN
PRODUCTION.

## What is implemented

Only three services are implemented:

- Local OAuth 2.0 authentication in REST;
- External OAuth 2.0 authentication in REST;
- User CRUD;
