#!/bin/sh
echo "Waiting for Vault..."
sleep 3

echo "Vault Started."

echo "Authenticate into Vault"
# Authenticate to Vault
vault login $VAULT_TOKEN

echo "Adding secrets to Vault..."
vault kv put -mount=secret HUGGINGFACE_API_TOKEN api_key=$HUGGINGFACE_API_TOKEN
vault kv put -mount=secret OPENAI_API_TOKEN api_key=$OPENAI_API_TOKEN