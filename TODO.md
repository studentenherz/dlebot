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
            -   [x] Dictionary
                -   [x] `get_list` -> `get_list_like`
                -   [x] `get_definition` -> `get_exact`
                -   [x] `get_random`
                -   [x] `get_word_of_the_day`
            -   [ ] Usage
                -   [ ] Events
                    -   [x] Create migrations and models
                    -   [ ] Handle the events
                -   [ ] Users
                    -   [x] Create migrations and models
                    -   [ ] Handle users
    -   [x] Find definition form message
    -   [x] Implement smart split of long messages
    -   [x] Implement commands related to DB
        -   [x] `/aleatorio`
        -   [x] `/pdd`
        -   [x] `/suscripcion`
    -   [x] Implement inline handler
    -   [ ] Implement schedule

    -   [ ] Handler possible errors

#### New features

-   [ ] Handle edited messages
-   [ ] Fuzzy search
-   [ ] Conjugation
-   [ ] Add other dictionaries
-   [ ] (Far into the future) inverse search
