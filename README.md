# A remote LLM for the LLM Verification Plugin for CTFd

This is meant to simplify and unify the API intefaces of the different APIs used to create chatbots with LLMs for competitions and assessments. It is currently designed to work in tandem with the [llm_verification](https://github.com/aivillage/llm_verification) plug-in for CTFd. 

# Roadmap

**Objectives:**

- Simplify and unify the interface across various LLMs
- Seperate keys and other important information from the competition's endpoint
- Provide idempotency so that the competition endpoint can safely retry
- Minimize network traffic

**Non-Objectives**

- Be a universal LLM API
- Host models

For V1:
- [ ] Vault Secret API Holding
- [ ] Monitoring of Models
- [ ] Rate limits for each model
- [ ] More LLM integrations

At V1:
- [ ] Production docker image 

# Development

To launch a dev instance `docker compose -f dockerfiles/docker-compose.dev.yml up --build`

To run tests `docker compose -f tests/docker-compose.yml run llm_router_test`xx