# Workload schema

Example :

```json
{
    "name": "test-workload",
    "type": "container",
    "executable": "nginx",
    "ressources": {
        "cpu": 1,
        "ram": 2048,
        "disk": 2,
    },
    "ports": [
        "8080:80"
    ]
}
```
## name

**type :** string

Name of the workload, should be unique in the controller’s database.

## type

**type :** string

Type of the workload :

- `"container"`
- `"vm"`
- `"binary"`

## executable

**type :** string

<!-- maybe change name -->

The name of the container image , or an URI to the vm image or binary executable.

## ressources

**type :** object/struct

Definition of the maximum (container, binary) or allocated (vm) ressources.

### ressources.cpu

**type :** TDB

CPU power, unit is to be defined.

### ressources.ram

**type :** integer

Memory amount, unit is MB.

### ressources.disk

**type :** integer

Disk size, unit is GB.

## ports

**type :** Array\<string\>

List of port mapping with the format `"external:internal"`.
