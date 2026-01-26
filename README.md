# rust-discord-bot
Just a random discordbot to learn rust with.


## Notes

This bot isn't really meant to be ran by anyone else. It's simply a for-fun bot for myself and my friends.

View docker-compose.example.yml for variables that are needed for this project.

The underlying filesystem is setup in a certain way for STATIC to work correctly with other docker containers.

Several commands related to AI won't run correctly out of the box.

Feature set is mostly what I find useful with a few for fun commands here and there. There's a lot of bloat with some of the open source bots, and I wanted to craft something efficient and not too overwhelming with the amount of commands there are.


## Features


### AI

- It uses a local ollama instance in order to make prompts when @mentioning the bot.
- Currently specifically created with gemma3 in mind, but could be expanded in the future.
- Stable Diffisuion with a self hosted instance (Currently not available)

### Anime
- Guess the OP
- Some integrations with [ShokoServer](https://github.com/ShokoAnime/ShokoServer/)
- Some integrations with Sonarr

### EmbedFix
- Fix embeds for instagram, reddit, twitter.

### Language
- Kanji search

### Music
(Currently broken)

### Permission system
- Basic permission system of admin/mod/trusted.
- Currently just used to prevent users from using certain commands that can be destructive.

### Random
- ping/bing
- avatar
- uptime
- userinfo
- guildinfo
- randomfox
- weather

### AYDY (Are you dead yet?)
- Based on the chinese app, where a message will be shown every 24 hours to the group chat. People can enroll, and if they're enrolled but dont' click the button for 48 hours, it will mark them in a separate table.

### Response System
- Based on a toml config file (/app/config.toml), you can create various different reactions you would like the bot to have.

### Stream Following
- Sends a message when a user is streaming. Currently only kick support.

### Ticketing System
WIP

### TMDB
- Basic support to search movies and tv shows.

### Papers
- A system to create a persistent embed message that can be used to assign roles on click, or can be used as a generic modifiable embedded message.

### Bot Profile
- Commands to help with modifying the bot profile.

### Tags
- Essentially a fancy discord key-value store.

### Website
- There is also a website that gets hosted on port 8080, that will have some nice tools for administrators like list of emojis, tags, commands, etc.