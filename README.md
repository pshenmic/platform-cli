# platform-cli v0.1.0

A binary application to create, sign and broadcast Dash Platform state transition from your computer.

Currently supported these types of actions:

* Credit Withdrawals
* Register a name
* Masternode vote for a contested resource

# Install

Download a binary from the Releases section on the GitHub and place it in the $PATH variable

# Usage

You can use --help to get more info about the flags 

Example commands:
```bash
$ platform-cli withdraw --dapi-url https://127.0.0.1:1443 --identity A1rgGVjRGuznRThdAA316VEEpKuVQ7mV8mBK1BFJvXnb --private-key private_key.txt --withdrawal-address yifJkXaxe7oM1NgBDTaXnWa6kXZAazBfjk --amount 40000
$ platform-cli register-dpns-name --identity 8eTDkBhpQjHeqgbVeriwLeZr1tCa6yBGw76SckvD1cwc --private-key private_key.txt --dapi-url https://52.43.13.92:1443 --name tesstst32423sts
$ platform-cli masternode-vote-dpns-name --dapi-url https://52.43.13.92:1443 --private-key voting_key.txt --pro-tx-hash 7a1ae04de7582262d9dea3f4d72bc24a474c6f71988066b74a41f17be5552652 --normalized-label testc0ntested --choice 8eTDkBhpQjHeqgbVeriwLeZr1tCa6yBGw76SckvD1cwc
```