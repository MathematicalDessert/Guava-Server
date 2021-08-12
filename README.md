# Guava-Server
Guava-Server is a content provider server for Guava clients.

## What is Guava?
Guava is a Client-Sided Roblox Content Provider. It allows for users to load in custom content on the client for local usage. Most content must go through Roblox moderation which can slow down the development and actively prevents the existence of certain interesting use-cases.

## How does Guava work?
Guava consists of two components. A content provider ([Guava-Server](https://github.com/MathematicalDessert/Guava-Server)) and a client ([Guava-Client](https://github.com/MathematicalDessert/Guava-Client)).

- Client - Downloads content from a content provider server and places said content in `roblox\versions\<client or studio version hash>\content\custom-content` with a manifest. Roblox clients (Player & Studio) can then load this data through fixed names for content: `rbxasset://custom-content/<content from server hash>`.

- Server - The server consists of `seeds` which are pieces of content which have a `type`, `hash`, `name`, and the actual `data`. These can be stored in `playlists` which are collections of seeds. The client can then choose to load in the playlist it needs to play a certain game or experience some content.

## Why is it called Guava?
Because it's f---ing sweet! :D
