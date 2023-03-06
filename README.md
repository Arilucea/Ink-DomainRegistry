# Substrate-DomainRegistry

## Deploy contract
cargo contract instantiate --constructor new --args "false" --suri //Alice --salt $(date +%s)

## Call getter (--dry-run sirve para asegurar que no ejecuta nada, solo lee)
cargo contract call --contract 5GRAVvuSXx8pCpRUDHzK6S1r2FjadahRQ6NEgAVooQ2bB8r5 --message get --suri //Alice --dry-run

## Call setter
cargo contract call --contract 5GQwxP5VTVHwJaRpoQsK5Fzs5cERYBzYhgik8SX7VAnvvbZS --message flip --suri //Alice