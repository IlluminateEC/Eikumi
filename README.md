# Eikumi

Eikumi is a moderation discord bot that was created for the Illuminate Engineering Corps discord server.

It handles moderation transparency along with implementing a community score system for users with automatic actions being applied depending on this score.

Written in Rust because Kazani wanted to try `serenity`.

## Setup

### `.env`

As of now, `.env` should only contain one environment variable:

```bash
DISCORD_TOKEN=abc123kfdhogh.45uf9hgnhg
```

### `config.json`

The config should contain a map of guild IDs to maps of channel types to channel ids.

```json
{
    "$$GUILD_ID$$": {
        "transparency": $$CHANNEL_ID$$,
        "membership": $$CHANNEL_ID$$,
    }
}
```