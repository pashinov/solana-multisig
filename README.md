# Solana multisig wallet

Implementation of a multisig wallet on solana. This program allows to create an associated multisig
account for a given wallet address and specify set of owners who can approve multisig transactions.
When number of approvals reaches threshold transaction will be executed.

## How to use

### Deploy the on-chain program

```bash
./run.sh deploy
```

### Create multisig account

```bash
./run.sh client create-account <THRESHOLD> <OWNER_PUBKEY_1> <OWNER_PUBKEY_2> ...
```

### Create transaction

```bash
./run.sh client create-transaction <RECIPIENT> <AMOUNT>
```

### Approve transactions related to multisig account

```bash
./run.sh client approve <MULTISIG>
```
