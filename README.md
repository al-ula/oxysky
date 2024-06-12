# OXYSKY

## Bluesky client in rust

### This is me learning rust

Only testing for now

## Available API

| Bluesky                          | Oxysky                             |
| -------------------------------- | ---------------------------------- |
| com.atproto.server.createSession | oxysky_lib::server::CreateSession.send() |
| com.atproto.server.deleteSession | oxysky_lib::server::Session.delete() |
| com.atproto.server.getSession | oxysky_lib::server::Session.get() |
| com.atproto.server.refreshSession | oxysky_lib::server::Session.refresh() |