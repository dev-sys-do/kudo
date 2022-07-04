# CLI configuration

This file describes the ways you can change the configuration of the client.

The config file is a yaml file located by default in the home directory of the user : `~/kudo/config.yaml`. There will be more option as the features grow.

## Config file location

**ENV :** `KUDO_CONFIG=~/kudo/config.yaml`

The config file location can only be changed via an environment variable. The value is a path to the file.

## Controller URL

**ENV :** `KUDO_CONTROLLER_URL="https://localhost:6443"`

**CONFIG FILE :**

```yaml
controller: 
  url: "https://localhost:6443"
```

URL of the controller api, the format is a normal URL : `<protocol>://<address>:<port>`.
