# async-discord

A (currently-not-yet) friendly Discord client library built for bot developers. Built for `async/await`, not ready for production.

The main design goal is to make it accessible to build more complex functionality via middleware traits.

Ideas borrowed and `twilight-model` used from [twilight-rs/twilight](https://github.com/twilight-rs/twilight).

Check out our [/examples](/examples). (api subject to massive change!)

---

### Choices

- `async-std` (may allow for other runtimes in the future.)
- `surf` (http)
- `async-tungstenite` / `async-native-tls` (websockets)

### Non-goals

- Voice
