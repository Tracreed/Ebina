# Ebina

## About The Project

Ebina is a multi-purpose Discord bot written in Rust, named after the character Nana Ebina from Himouto! Umaru-chan. It was initially created to play charades (naming an anime/series from a set of emotes) but has since been expanded to include a variety of other features.

The bot features a web dashboard for control and a Prometheus endpoint for live command metrics. It's designed for easy deployment using Docker and uses a PostgreSQL database for persistent storage of configuration and charades data.

## Features

Ebina integrates with a wide array of external services to provide a rich user experience.

### Connected APIs:
*   **WolframAlpha**: For computational knowledge.
*   **OpenWeatherMap**: For weather forecasts.
*   **Anilist**: For anime and manga tracking.
*   **SauceNAO**: For reverse image searches (with image upload support).
*   **VNDB.org**: For Visual Novel database lookups.
*   **Mangadex.org**: For manga information.
*   **Osu!**: For rhythm game stats.
*   **Helium API**: For the Helium blockchain.

## Getting Started

To get a local copy up and running follow these simple steps.

### Prerequisites

*   [Docker](https://www.docker.com/)
*   [Rust](https://www.rust-lang.org/)

### Installation

1.  Clone the repo
    ```sh
    git clone https://github.com/your_username_/Ebina.git
    ```
2.  Create a `.env` file in the `ebina-bot` directory and add the required environment variables. See the Configuration section for more details.
3.  Build and run the Docker container:
    ```sh
    docker-compose up --build
    ```

## Usage

Once the bot is running, you can interact with it in your Discord server. For example, to get the weather for a specific location, use the `!weather` command:

```
!weather London
```

## Configuration

Create a `.env` file in the `ebina-bot` directory with the following variables:

| Variable          | Description                                   |
| ----------------- | --------------------------------------------- |
| `DISCORD_TOKEN`   | Your Discord bot token.                       |
| `PREFIX`          | The command prefix for the bot (e.g., `!`).   |
| `APPLICATION_ID`  | Your Discord application ID.                  |
| `OSU_ID`          | Your Osu! API client ID.                      |
| `OSU_SECRET`      | Your Osu! API client secret.                  |
| `DATABASE_URL`    | The connection URL for your PostgreSQL database. |
| `WEATHER_KEY`     | Your OpenWeatherMap API key.                  |
| `WOLFRAM_ALPHA`   | Your WolframAlpha API key.                    |
| `SAUCENAO`        | Your SauceNAO API key.                        |

## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request
