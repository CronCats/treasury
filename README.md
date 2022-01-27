# treasury
Autonomous treasury for enabling DAOs or any type of near account to manage multiple funds to accrue interest across different vehicles

----

## WARNING!!

This repo is under HEAVY Development, should not be considered ready for any type of use. It is pre-alpha!

----

### Development

**Build & Dev**

```bash
./build.sh
```

**Testing**

```bash
./test.sh

# OR
cargo run --e2e treasury
```

**Bootstrapping**

```bash
## Setup
./scripts/clear_all.sh
./scripts/create_and_deploy.sh
./scripts/bootstrap.sh

## Test features, by flow:
./scripts/staking.sh
./scripts/actions.sh
```

### Notes:

* [Contract ABI](./ABI.md)
* [Features & Flows](./FLOWS.md)