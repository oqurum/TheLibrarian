# IMPORTANT
**Recently ported from sqlite to postgres. Will probably contain invalid SQL queries**


# Welcome
This repo will ultimately be for the main metadata website we use for the client application. It will be hosted on the official website. I hope this can be one day used to aggregate the other metadata agents into here.


You can view the public website below:

**[https://oqurum.io](https://oqurum.io)**

You can also log into it:
 - Email: viewer@oqurum.io
 - Pass: password123


Todo:
 - **#1 Priority: Implement Full-Text Search**
 - Implement database migrations
 - Member management
 - Allow users to upload drafts of their own to this server if they're linked.
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
Execute one of these commands

To build:
```bash
cd crates/frontend
trunk build --public-url "/dist" -d "../../app/public/dist"
```

To build and watch:
```bash
cd crates/frontend
trunk watch --public-url "/dist" -d "../../app/public/dist"
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