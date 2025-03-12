# Profile Microservice

## Getting started

### Installation

```sh
cargo install --git https://github.com/ferristhecrab/atom-profile
```

You **must** compile profiles with The following feature flags to run is as an executable
- core (required)
- services-core or services-request (choose one)

### Running

#### Prerequisite
MongoDB running with [authentication set up](https://www.geeksforgeeks.org/how-to-enable-authentication-on-mongodb/);

```sh
CONFIG=/home/yourname/.config/atomics/profile.json atom-profile
```

Where `CONFIG` can be replaced with the location to the config file.

## API

Schema definition in [schema](./src/schema), exposed struct `Router` and `InternalRouter` in [router.rs](./src/router.rs) for squashed microservices.

