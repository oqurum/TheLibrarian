# Welcome

Todo:
 - REDO SCSS and move into common git repo to share with reader
 - Store external searches, count, and display most requested.
 - Add task to search agents for new external searchs as to not overwhelm agents
 - and more in TODO comments throughout the code


# Running/Building

To run and build the application you need to do the following:

[Install Rust](https://www.rust-lang.org/). It's used for coding this whole application.

[Install Trunk](https://trunkrs.dev/#install). It's used for building the frontend.

## Backend:
Inside **root folder** execute these commands:
```bash
cargo run --bin librarian-backend
```

## Frontend:
Inside **frontend folder** execute one of these commands

To build:
```bash
trunk build --public-url "/dist" -d "../app/public/dist"
```

To build and watch:
```bash
trunk watch --public-url "/dist" -d "../app/public/dist"
```


# Gallery

## List of books stored.
![Home](https://i.thick.at/NotableNewRadicals898.png)

## Book Viewer.
![Book View](https://i.thick.at/UbiquitarianBoston040.png)

## Book Editor
![Book Editor](https://i.thick.at/PhanerogamousKatherine428.png)

## Author Editor
![Author Viewer](https://i.thick.at/GoodTemperedDrangsal928.png)

## Edits
![Book Reader With Options](https://i.thick.at/ButyraceousMantaRay091.png)