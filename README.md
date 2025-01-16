# Eikumi

Eikumi is a moderation discord bot that was created for the Illuminate Engineering Corps discord server.

It handles moderation transparency along with implementing a community score system for users with automatic actions being applied depending on this score.

## Setup

### `.env`

Your `.env` file should contain:
```bash
DISCORD_TOKEN=YOUR.TOKEN.HERE
DATABASE_URL=postgres://eikumi:potato@127.0.0.1:47582/eikumi  # an example.
```

### `config.json`

The config should contain a map of guild IDs to maps of channel types to channel ids.

```json
{
    "$$GUILD_ID$$": {
        "transparency_channel": $$CHANNEL_ID$$,
        "membership_channel": $$CHANNEL_ID$$,
    }
}
```