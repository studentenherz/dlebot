<div align=center>
<img src="img/rusty-transparent.webp" width=280px/>
<h1><a href="https://github.com/studentenherz/dleraebot">@dlebot</a> gets Rusty</h1>
</div>

This is a new, rusty, iteration of <a href="https://github.com/studentenherz/dleraebot">studentenherz/dleraebot</a> that was originally developed in Python. For some time now I've been meaning to add some new features to the bot, but, as the time when I developed de bot was falling further into the past, the implementation and configuration details became almost arcane technology.

With over a year of experience since the [last commit](https://github.com/studentenherz/dleraebot/tree/319a4056b54ce2b0a889cf76677acbd6f309e7b6) I'm very confident that I'm able to rewrite the project in Python taking better practices into account, making it easier to maintain and add features. However, instead of doing that, I'm going to develop this other version of th bot using Rust for two main reasons:

-   Rust is faster than Python (although in this case I'm not sure it will make a noticeable difference);
-   I want to. I want to learn Rust with a project and this will be it.

I guess that, as this will be a learning project, in the end I'll get something that in a year from now I'll see in the same way I see the previous version.

## Setup

1. Clone this repo.

    ```sh
    git clone https://github.com/studentenherz/dlebot.git
    ```

2. Set up environment variables. See [`.env-sample`](./.env-sample) for the list of required and optional environment variables. You can set them or use a `.env` file based on [`.env-sample`](./.env-sample).

    There you can set up a custom API url if you are using a local instance of [telegram-bot-api](https://github.com/tdlib/telegram-bot-api); there is no need for this, but the Telegram BOT API servers are located in Europe, depending on where you are running the bot, it might be better to run your own instance.

3. Start the database

    ```sh
    docker compose up -d
    ```

4. Run the migrations, read [here](./migration/README.md) in order to see more detail.

    ```sh
    cd migration
    cargo run
    ```

5. Populate the database from a .csv with the columns in the order `lemma, definition, conjugation` (or specify the order in the command, see [here](https://www.postgresql.org/docs/current/sql-copy.html)).

    ```sh
    cat dle.csv | psql $DATABASE_URL -c 'COPY dle FROM STDIN (FORMAT csv)'
    ```

6. For development run the bot with

    ```sh
    cargo run
    ```

    For production use the `--release` option.

7. To install in systemd run the script [`install.sh`](./install.sh) from the root of the repo. Optionally, if you have a local instance of the BOT API you can install it to systemd with [`install-telegram-bot-api.sh`](./install-telegram-bot-api.sh)
