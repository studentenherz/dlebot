# Running Migrator CLI

First of all set up environment variables. See [`.env-sample`](./.env-sample) for the list of required and optional environment variables. You can set them or use a `.env` file based on [`.env-sample`](./.env-sample).

-   Generate a new migration file
    ```sh
    cargo run -- generate MIGRATION_NAME
    ```
-   Apply all pending migrations
    ```sh
    cargo run
    ```
    ```sh
    cargo run -- up
    ```
-   Apply first 10 pending migrations
    ```sh
    cargo run -- up -n 10
    ```
-   Rollback last applied migrations
    ```sh
    cargo run -- down
    ```
-   Rollback last 10 applied migrations
    ```sh
    cargo run -- down -n 10
    ```
-   Drop all tables from the database, then reapply all migrations
    ```sh
    cargo run -- fresh
    ```
-   Rollback all applied migrations, then reapply all migrations
    ```sh
    cargo run -- refresh
    ```
-   Rollback all applied migrations
    ```sh
    cargo run -- reset
    ```
-   Check the status of all migrations
    ```sh
    cargo run -- status
    ```
