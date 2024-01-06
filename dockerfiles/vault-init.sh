#!/bin/sh
echo "This is a development image for testing Vault secrets. Do not use in production."
echo "Waiting for the development Vault to sufficiently start..."
sleep 3

echo "Authenticate into Vault"
# Authenticate to Vault
vault login $VAULT_TOKEN

echo "Adding secrets to Vault..."
vault kv put -mount=secret HUGGINGFACE_API_TOKEN api_key=$HUGGINGFACE_API_TOKEN
vault kv put -mount=secret OPENAI_API_TOKEN api_key=$OPENAI_API_TOKEN