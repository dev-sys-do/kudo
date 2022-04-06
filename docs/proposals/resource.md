# Resource schema

A schema that can be sent to the controller in order to create a ressource.

## name

**type :** string  

Name of the ressource.

## type

**type :** string

Type of the ressource :

- `container` : see [workload.md](./workload.md).
- `vm` : see [workload.md](./workload.md).
- `binary`: see [workload.md](./workload.md).
- `user` : will be used for the creation of an user, the definition is not expected in v0.
