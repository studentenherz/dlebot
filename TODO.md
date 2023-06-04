## ToDo

-   [ ] Reproduce bot in Rust
    -   [x] Learn how to handle updates
    -   [x] Handle commands that don't require database connection
        -   [x] `/start`
        -   [x] `/help`
            -   [x] Add inline button for `/help`
    -   [x] Add keyboard buttons
    -   [ ] Database
        -   [x] Create migrations
        -   [x] Connect to database
        -   [x] Models
        -   [ ] Handlers
            -   [ ] Dictionary
                -   [x] `get_list` -> `get_list_like`
                -   [x] `get_definition` -> `get_exact`
                -   [x] `get_random`
                -   [ ] `get_word_of_the_day`
            -   [ ] Usage
                -   [ ] `add_message`
                -   [ ] `add_query`
                -   [ ] `set_block`
                -   [ ] `set_in_bot`
                -   [ ] `is_subscribed`
                -   [ ] `get_users_ids`
                -   [ ] `get_users_count`
                -   [ ] `get_usage_last`
    -   [x] Find definition form message
    -   [x] Implement smart split of long messages
    -   [ ] Implement commands related to DB
        -   [x] `/aleatorio`
        -   [ ] `/pdd`
        -   [ ] `/suscripcion`
    -   [ ] Implement inline handler
    -   [ ] Implement schedule

#### New features

-   [ ] Handle edited messages
-   [ ] Conjugation
