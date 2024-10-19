# platform-cli v0.1.3

A binary application to create, sign and broadcast Dash Platform state transition from your computer.

Application designed to provide an easy-to-use CLI terminal interface to create, 
sign, and broadcast Dash Platform state transitions by passing down all 
necessary input data through command flags and arguments

Currently, there is a support of 3 given actions:

* Credits Withdrawal
* Register a name
* Masternode vote for a DPNS name

Other actions will be implemented in future version (listed in order of priority):

* Register Identity
* Create document
* Set price of the Document
* Purchase document
* Create data contract
* Masternode Vote Manual (for voting on any contested documents, no just DPNS one)

The tool initially built to allow pshenmic.dev DFO to make operations (JS Dash SDK is not feature-complete), but can be used by anyone to simply make transactions in the network without messing around with any UI interfaces.

Any nice or missing features can be proposed by anyone via submitting an issue in the GitHub repository

https://github.com/pshenmic/platform-cli/issues


## Install

1) Download a binary from the [Releases](https://github.com/pshenmic/platform-cli/releases) (matching your CPU architecture)
2) Rename it to `platform-cli`
3) Make it executable (`chmod +x platform-cli`)
4) Add it to your `$PATH` environment variable, or simply put in the `/usr/local/bin`

## Usage
Just execute a binary to run start working with an application.

You can use --help to get more info about command flags

Example commands:
```bash
$ platform-cli withdraw --dapi-url https://127.0.0.1:1443 --identity A1rgGVjRGuznRThdAA316VEEpKuVQ7mV8mBK1BFJvXnb --private-key private_key.txt --withdrawal-address yifJkXaxe7oM1NgBDTaXnWa6kXZAazBfjk --amount 40000
$ platform-cli register-dpns-name --identity 8eTDkBhpQjHeqgbVeriwLeZr1tCa6yBGw76SckvD1cwc --private-key private_key.txt --dapi-url https://52.43.13.92:1443 --name tesstst32423sts
$ platform-cli masternode-vote-dpns-name --dapi-url https://52.43.13.92:1443 --private-key voting_key.txt --pro-tx-hash 7a1ae04de7582262d9dea3f4d72bc24a474c6f71988066b74a41f17be5552652 --normalized-label testc0ntested --choice 8eTDkBhpQjHeqgbVeriwLeZr1tCa6yBGw76SckvD1cwc
```

### Credits Withdrawal
```bash
Withdraw credits from the Identity to the L1 Core chain

Usage: platform-cli withdraw [OPTIONS]

Options:
      --dapi-url <DAPI_URL>
          DAPI GRPC Endpoint URL, ex. https://127.0.0.1:1443 [default: ]
      --identity <IDENTITY>
          Identity address, that initiate withdrawal [default: ]
      --private-key <PRIVATE_KEY>
          Identity private key in WIF format [default: ]
      --withdrawal-address <WITHDRAWAL_ADDRESS>
          Core withdrawal address (P2PKH / P2SH) [default: ]
      --amount <AMOUNT>
          Amount of credits to withdraw [default: ]
  -h, --help
          Print help
```

After a successful transaction broadcast in the Platform netwowrk,
your payment will be placed in the queue in the Core chain waiting for a
specific quorum to come up to finish a withdrawal (technical limitation of Dash Core protocol)

There is no way to track it now, so in case you have successful transaction on Platform Explorer,
just wait for a funds to come up in your receiving wallet

### Register DPNS Name
```bash
Register an Identity Name in the Dash Platform DPNS system

Usage: platform-cli register-dpns-name [OPTIONS]

Options:
      --dapi-url <DAPI_URL>        DAPI GRPC Endpoint URL, ex. https://127.0.0.1:1443 [default: ]
      --identity <IDENTITY>        Identity address that registers a name [default: ]
      --private-key <PRIVATE_KEY>  Identity private key in WIF format [default: ]
      --name <NAME>                Name to register (excluding .dash) [default: ]
  -h, --help                       Print help
```

If your name falls under DPNS contested resource rules (`/^[a-zA-Z01-]{3,19}$/`),<br>
a prepaid balance of 0.2 DASH automatically added in the transaction, and
contested resource poll on your name automatically starts

### Masternode vote for DPNS name
```bash
Perform a masternode vote towards contested DPNS name

Usage: platform-cli masternode-vote-dpns-name [OPTIONS]

Options:
      --dapi-url <DAPI_URL>
          DAPI GRPC Endpoint URL, ex. https://127.0.0.1:1443 [default: ]
      --pro-tx-hash <PRO_TX_HASH>
          ProTxHash of the Masternode performing a Vote, in hex [default: ]
      --private-key <PRIVATE_KEY>
          Voting (or Owner) private key in WIF format [default: ]
      --normalized-label <NORMALIZED_LABEL>
          Normalized label to vote upon (can be grabbed from https//dash.vote) [default: ]
      --choice <CHOICE>
          The choice of the Vote. It can be an Identifier you are voting towards (ex. BMJWm8wKmbApR7nQ6q7RG3HgD8maJ8t7B4yWBKRe7aZ6), or Lock, or Abstain [default: ]
  -h, --help
          Print help
```


